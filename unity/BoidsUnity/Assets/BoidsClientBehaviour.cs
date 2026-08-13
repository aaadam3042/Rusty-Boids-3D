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
    
    private readonly ConcurrentQueue<string> snapshots = new();
    private readonly ConcurrentQueue<string> diagnostics = new();
    private readonly Dictionary<int, GameObject> boidsById = new();

    private Process hostProcess;
    private GameObject boidsEmpty;

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

            return Time.realtimeSinceStartupAsDouble -
                latestSnapshotAcceptedAt;
        }
    }

    private double latestSnapshotAcceptedAt = -1.0;
    private long rxDiscardedSinceLastLog;

    private int fpsFrameCount;
    private double fpsWindowStartedAt;
    private double unityFps;

    private double nextHealthLogAt;

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
        double now = Time.realtimeSinceStartupAsDouble;
        fpsWindowStartedAt = now;
        nextHealthLogAt = now + 1.0;

        boidsEmpty = new GameObject("Boids");
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

        try
        {
            Process process = new Process { StartInfo = startInfo };
            
            process.OutputDataReceived += (_, eventArgs) =>
            {
                if (!string.IsNullOrEmpty(eventArgs.Data))
                    snapshots.Enqueue(eventArgs.Data);
            };

            process.ErrorDataReceived += (_, eventArgs) =>
            {
                if (!string.IsNullOrEmpty(eventArgs.Data))
                    diagnostics.Enqueue(eventArgs.Data);
            };

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

        fpsFrameCount++;

        double fpsElapsed = now - fpsWindowStartedAt;

        if (fpsElapsed >= 1.0)
        {
            unityFps = fpsFrameCount / fpsElapsed;
            fpsFrameCount = 0;
            fpsWindowStartedAt = now;
        }

        string newestJson = null;
        int dequeuedSnapshots = 0;

        while (snapshots.TryDequeue(out string json))
        {
            newestJson = json;   
            dequeuedSnapshots++;
        }

        DiscardedSnapshotsLastFrame = dequeuedSnapshots > 0 ? dequeuedSnapshots - 1 : 0;
        TotalDiscardedSnapshots += DiscardedSnapshotsLastFrame;

        if (newestJson != null)
        {
            try
            {
                WorldSnapshot snapshot = JsonUtility.FromJson<WorldSnapshot>(newestJson);
                if (snapshot == null || snapshot.boids == null)
                {
                    Debug.LogWarning($"Invalid boids snapshot: {newestJson}");
                } 
                else
                {
                    LatestTick = snapshot.tick;
                    LatestHostHealth = snapshot.health;
                    latestSnapshotAcceptedAt = Time.realtimeSinceStartupAsDouble;

                    ApplySnapshot(snapshot);   
                }
                
                if (now >= nextHealthLogAt && LatestHostHealth != null)
                {
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
                    performanceDisplay.Render(LatestHostHealth, rxDiscardedSinceLastLog, TotalDiscardedSnapshots, unityFps);

                    rxDiscardedSinceLastLog = 0;
                    nextHealthLogAt = now + 1.0;
                }
            } 
            catch (Exception e)
            {
                Debug.LogWarning($"Unable to deserialise boids snapshot: {e.Message}");
            }
        }

        while (diagnostics.TryDequeue(out string message))
        {
            Debug.Log($"boids-host: {message}");
        }
    }

    private void ApplySnapshot(WorldSnapshot snapshot)
    {
        foreach (BoidSnapshot boidSnapshot in snapshot.boids)
        {
            if (!boidsById.TryGetValue(boidSnapshot.id, out GameObject boid)) 
            {
                boid = CreateBoid(boidSnapshot.id);
                boid.transform.SetParent(boidsEmpty.transform);
                boidsById.Add(boidSnapshot.id, boid);
            } 
            boid.transform.position = boidSnapshot.position;
        }
    }

    private GameObject CreateBoid(int boid_id)
    {
        GameObject boid = GameObject.CreatePrimitive(PrimitiveType.Sphere);
        boid.name = $"Boid {boid_id}";
        return boid;
    }
}
