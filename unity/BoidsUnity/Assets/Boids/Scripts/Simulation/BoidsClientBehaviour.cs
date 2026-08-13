using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using UnityEngine;
using Debug = UnityEngine.Debug;

public class BoidsClientBehaviour : MonoBehaviour
{
    [SerializeField] private string executablePath;
    [SerializeField] private PerformanceDisplay performanceDisplay;
    [SerializeField] private GameObject boidPrefab;

    private readonly ConcurrentQueue<string> snapshots = new();
    private readonly ConcurrentQueue<string> diagnostics = new();
    private readonly Dictionary<int, GameObject> boidsById = new();

    private Process hostProcess;
    private Transform boidsRoot;
    private double latestSnapshotAcceptedAt = -1.0;
    private long rxDiscardedSinceLastLog;
    private int fpsFrameCount;
    private double fpsWindowStartedAt;
    private double unityFps;
    private double nextHealthLogAt;

    public HostHealthSnapshot LatestHostHealth { get; private set; }
    public long LatestTick { get; private set; } = -1;
    public int DiscardedSnapshotsLastFrame { get; private set; }
    public long TotalDiscardedSnapshots { get; private set; }

    public double SecondsSinceLatestSnapshot
    {
        get
        {
            if (latestSnapshotAcceptedAt < 0.0)
                return double.PositiveInfinity;

            return Time.realtimeSinceStartupAsDouble - latestSnapshotAcceptedAt;
        }
    }

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        double now = Time.realtimeSinceStartupAsDouble;
        fpsWindowStartedAt = now;
        nextHealthLogAt = now + 1.0;

        boidsRoot = new GameObject("Boids").transform;
        boidsRoot.SetParent(transform, false);
        StartHost();
    }

    private void StartHost()
    {
        if (hostProcess != null && !hostProcess.HasExited) return;
        string path = Path.GetFullPath(executablePath);

        if (!File.Exists(path))
        {
            Debug.LogError($"boids-host was not found at {path}");
        }

        try
        {
            Process process = CreateHostProcess(path);

            bool started = process.Start();
            if (started)
            {
                Debug.Log($"boids-host started. PID: {process.Id}");
            }
            else
            {
                Debug.LogError("Process.Start() returned false.");
            }

            hostProcess = process;
            hostProcess.BeginOutputReadLine();
            hostProcess.BeginErrorReadLine();
        }
        catch (Exception e)
        {
            Debug.LogWarning($"Failed to start host process: {e.Message}");
        }
    }

    private Process CreateHostProcess(string path)
    {
        var startInfo = new ProcessStartInfo
        {
            FileName = path,
            WorkingDirectory = Path.GetDirectoryName(path),
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true
        };

        Process process = new Process { StartInfo = startInfo };
        process.OutputDataReceived += OnHostOutputReceived;
        process.ErrorDataReceived += OnHostErrorReceived;
        return process;
    }

    private void OnHostOutputReceived(object sender, DataReceivedEventArgs eventArgs)
    {
        if (!string.IsNullOrEmpty(eventArgs.Data))
            snapshots.Enqueue(eventArgs.Data);
    }

    private void OnHostErrorReceived(object sender, DataReceivedEventArgs eventArgs)
    {
        if (!string.IsNullOrEmpty(eventArgs.Data))
            diagnostics.Enqueue(eventArgs.Data);
    }

    void OnDestroy()
    {
        Debug.Log("OnDestroy called");
        StopHost();
    }

    private void StopHost()
    {
        Process process = hostProcess;
        hostProcess = null;

        if (process == null) return;

        try
        {
            if (!process.HasExited)
            {
                // boids-host should interpret this as a graceful shutdown.
                process.StandardInput.WriteLine("{\"type\":\"shutdown\"}");
                process.StandardInput.Flush();
                process.StandardInput.Close();

                if (!process.WaitForExit(1000))
                    process.Kill();
            }
        }
        catch (Exception exception)
        {
            Debug.LogWarning($"Could not stop boids-host cleanly: {exception.Message}");
        }
        finally
        {
            process.Dispose();
        }
    }

    // Update is called once per frame
    void Update()
    {
        double now = Time.realtimeSinceStartupAsDouble;

        UpdateFrameRate(now);
        ProcessNewestSnapshot(now);
        DrainDiagnostics();
    }

    private void UpdateFrameRate(double now)
    {
        fpsFrameCount++;
        double fpsElapsed = now - fpsWindowStartedAt;

        if (fpsElapsed < 1.0) return;

        unityFps = fpsFrameCount / fpsElapsed;
        fpsFrameCount = 0;
        fpsWindowStartedAt = now;
    }

    private void ProcessNewestSnapshot(double now)
    {
        string newestJson = DequeueNewestSnapshot();
        if (newestJson == null) return;

        try
        {
            WorldSnapshot snapshot = JsonUtility.FromJson<WorldSnapshot>(newestJson);

            if (snapshot == null || snapshot.boids == null)
            {
                Debug.LogWarning($"Invalid boids snapshot: {newestJson}");
            }
            else
            {
                AcceptSnapshot(snapshot);
            }

            RenderPerformanceDisplay(now);
        }
        catch (Exception e)
        {
            Debug.LogWarning($"Unable to deserialise boids snapshot: {e.Message}");
        }
    }

    private string DequeueNewestSnapshot()
    {
        string newestJson = null;
        int dequeuedSnapshots = 0;

        while (snapshots.TryDequeue(out string json))
        {
            newestJson = json;
            dequeuedSnapshots++;
        }

        DiscardedSnapshotsLastFrame = dequeuedSnapshots > 0 ? dequeuedSnapshots - 1 : 0;
        TotalDiscardedSnapshots += DiscardedSnapshotsLastFrame;
        return newestJson;
    }

    private void AcceptSnapshot(WorldSnapshot snapshot)
    {
        LatestTick = snapshot.tick;
        LatestHostHealth = snapshot.health;
        latestSnapshotAcceptedAt = Time.realtimeSinceStartupAsDouble;
        ApplySnapshot(snapshot);
    }

    private void RenderPerformanceDisplay(double now)
    {
        if (now < nextHealthLogAt || LatestHostHealth == null) return;

        // Debug.Log(
        //     $"SIM {LatestHostHealth.realTimeFactor:F2}x | " +
        //     $"tick {LatestTick} | " +
        //     $"late {LatestHostHealth.deadlineLatenessMs:F2} ms | " +
        //     $"step {LatestHostHealth.lastStepMs:F2} ms | " +
        //     $"publish {LatestHostHealth.previousPublishMs:F2} ms | " +
        //     $"RX discarded {rxDiscardedSinceLastLog} " +
        //     $"({TotalDiscardedSnapshots} total) | " +
        //     $"Unity {unityFps:F1} FPS"
        // );
        performanceDisplay.Render(
            LatestHostHealth,
            rxDiscardedSinceLastLog,
            TotalDiscardedSnapshots,
            unityFps);

        rxDiscardedSinceLastLog = 0;
        nextHealthLogAt = now + 1.0;
    }

    private void DrainDiagnostics()
    {
        while (diagnostics.TryDequeue(out string message))
        {
            Debug.Log($"boids-host: {message}");
        }
    }

    private void ApplySnapshot(WorldSnapshot snapshot)
    {
        foreach (BoidSnapshot boidSnapshot in snapshot.boids)
        {
            GameObject boid = GetOrCreateBoid(boidSnapshot.id);
            boid.transform.position = boidSnapshot.position;
        }
    }

    private GameObject GetOrCreateBoid(int boidId)
    {
        if (boidsById.TryGetValue(boidId, out GameObject boid))
            return boid;

        boid = CreateBoid(boidId);
        boidsById.Add(boidId, boid);
        return boid;
    }

    private GameObject CreateBoid(int boid_id)
    {
        GameObject boid = Instantiate(boidPrefab, boidsRoot, false);
        boid.name = $"Boid {boid_id}";
        return boid;
    }
}
