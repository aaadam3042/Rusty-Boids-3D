using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using UnityEngine;
using Debug = UnityEngine.Debug;

public class BoidsClientBehaviour : MonoBehaviour
{
    private const double ReadyTimeoutSeconds = 5.0;

    [SerializeField] private string executablePath;
    [SerializeField] private PerformanceDisplay performanceDisplay;
    [SerializeField] private GameObject boidPrefab;
    [SerializeField] private BoidsBoundsView boundsView;

    private readonly ConcurrentQueue<string> protocolMessages = new();
    private readonly ConcurrentQueue<string> diagnostics = new();
    private readonly Dictionary<int, GameObject> boidsById = new();

    private Process hostProcess;
    private Transform boidsRoot;
    private double readyDeadline;
    private double helloRetryAt;
    private bool helloRetried;
    private double latestSnapshotAcceptedAt = -1.0;
    private long rxDiscardedSinceLastLog;
    private int fpsFrameCount;
    private double fpsWindowStartedAt;
    private double unityFps;
    private double nextHealthLogAt;

    public BoidsConnectionState ConnectionState { get; private set; } = BoidsConnectionState.Stopped;
    public BoundsSnapshot LatestBounds { get; private set; }
    public WeightsSnapshot LatestWeights { get; private set; }
    public HostHealthSnapshot LatestHostHealth { get; private set; }
    public long LatestTick { get; private set; } = -1;
    public int DiscardedSnapshotsLastFrame { get; private set; }
    public long TotalDiscardedSnapshots { get; private set; }

    public bool TryGetBoidTransform(int boidId, out Transform boidTransform)
    {
        if (boidsById.TryGetValue(boidId, out GameObject boid))
        {
            boidTransform = boid.transform;
            return true;
        }

        boidTransform = null;
        return false;
    }

    public bool SetWeights(float cohesion, float alignment, float separation)
    {
        if (ConnectionState != BoidsConnectionState.Ready)
        {
            Debug.LogWarning("Cannot set boid weights before boids-host is ready.", this);
            return false;
        }

        return SendCommand(new SetWeightsCommand(cohesion, alignment, separation));
    }

    public double SecondsSinceLatestSnapshot
    {
        get
        {
            if (latestSnapshotAcceptedAt < 0.0)
                return double.PositiveInfinity;

            return Time.realtimeSinceStartupAsDouble - latestSnapshotAcceptedAt;
        }
    }

    private void Start()
    {
        double now = Time.realtimeSinceStartupAsDouble;
        fpsWindowStartedAt = now;
        nextHealthLogAt = now + 1.0;

        boidsRoot = new GameObject("Boids").transform;
        boidsRoot.SetParent(transform, false);

        if (boundsView == null)
            boundsView = GetComponent<BoidsBoundsView>();
        if (boundsView == null)
            boundsView = gameObject.AddComponent<BoidsBoundsView>();

        StartHost(now);
    }

    private void StartHost(double now)
    {
        if (hostProcess != null && !hostProcess.HasExited)
            return;

        ConnectionState = BoidsConnectionState.Starting;
        string path = Path.GetFullPath(executablePath);

        if (!File.Exists(path))
        {
            FaultConnection($"boids-host was not found at {path}");
            return;
        }

        try
        {
            Process process = CreateHostProcess(path);
            if (!process.Start())
            {
                process.Dispose();
                FaultConnection("Process.Start() returned false for boids-host.");
                return;
            }

            hostProcess = process;
            hostProcess.BeginOutputReadLine();
            hostProcess.BeginErrorReadLine();

            ConnectionState = BoidsConnectionState.AwaitingReady;
            readyDeadline = now + ReadyTimeoutSeconds;
            helloRetryAt = now + ReadyTimeoutSeconds / 2.0;
            helloRetried = false;

            if (!SendCommand(new HelloCommand()))
                return;

            Debug.Log($"boids-host started. PID: {process.Id}", this);
        }
        catch (Exception exception)
        {
            FaultConnection($"Failed to start boids-host: {exception.Message}");
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

        Process process = new() { StartInfo = startInfo };
        process.OutputDataReceived += OnHostOutputReceived;
        process.ErrorDataReceived += OnHostErrorReceived;
        return process;
    }

    private void OnHostOutputReceived(object sender, DataReceivedEventArgs eventArgs)
    {
        if (!string.IsNullOrEmpty(eventArgs.Data))
            protocolMessages.Enqueue(eventArgs.Data);
    }

    private void OnHostErrorReceived(object sender, DataReceivedEventArgs eventArgs)
    {
        if (!string.IsNullOrEmpty(eventArgs.Data))
            diagnostics.Enqueue(eventArgs.Data);
    }

    private void OnApplicationQuit()
    {
        StopHost();
    }

    private void OnDestroy()
    {
        StopHost();
    }

    private void StopHost()
    {
        Process process = hostProcess;
        hostProcess = null;

        if (process == null)
        {
            if (ConnectionState != BoidsConnectionState.Faulted)
                ConnectionState = BoidsConnectionState.Stopped;
            return;
        }

        try
        {
            if (!process.HasExited)
            {
                string json = JsonUtility.ToJson(new ShutdownCommand());
                process.StandardInput.WriteLine(json);
                process.StandardInput.Flush();
                process.StandardInput.Close();

                if (!process.WaitForExit(1000))
                    process.Kill();
            }
        }
        catch (Exception exception)
        {
            Debug.LogWarning($"Could not stop boids-host cleanly: {exception.Message}", this);
        }
        finally
        {
            process.Dispose();
        }

        if (ConnectionState != BoidsConnectionState.Faulted)
            ConnectionState = BoidsConnectionState.Stopped;
    }

    private void Update()
    {
        double now = Time.realtimeSinceStartupAsDouble;

        UpdateFrameRate(now);
        ProcessProtocolMessages(now);
        DrainDiagnostics();
        CheckReadyTimeout(now);
        CheckForUnexpectedHostExit();
    }

    private void UpdateFrameRate(double now)
    {
        fpsFrameCount++;
        double fpsElapsed = now - fpsWindowStartedAt;

        if (fpsElapsed < 1.0)
            return;

        unityFps = fpsFrameCount / fpsElapsed;
        fpsFrameCount = 0;
        fpsWindowStartedAt = now;
    }

    private void ProcessProtocolMessages(double now)
    {
        string newestSnapshotJson = null;
        int receivedSnapshots = 0;

        while (protocolMessages.TryDequeue(out string json))
        {
            try
            {
                HostMessageHeader header = JsonUtility.FromJson<HostMessageHeader>(json);
                if (header == null || string.IsNullOrEmpty(header.type))
                {
                    Debug.LogWarning($"boids-host sent a message without a type: {json}", this);
                    continue;
                }

                switch (header.type)
                {
                    case "ready":
                        HandleReady(JsonUtility.FromJson<ReadyMessage>(json));
                        break;
                    case "snapshot":
                        newestSnapshotJson = json;
                        receivedSnapshots++;
                        break;
                    case "weightsUpdated":
                        HandleWeightsUpdated(JsonUtility.FromJson<WeightsUpdatedMessage>(json));
                        break;
                    case "error":
                        HandleProtocolError(JsonUtility.FromJson<ProtocolErrorMessage>(json));
                        break;
                    default:
                        Debug.LogWarning($"boids-host sent unknown message type '{header.type}'.", this);
                        break;
                }
            }
            catch (Exception exception)
            {
                Debug.LogWarning($"Unable to deserialise boids-host message: {exception.Message}", this);
            }
        }

        DiscardedSnapshotsLastFrame = receivedSnapshots > 0 ? receivedSnapshots - 1 : 0;
        TotalDiscardedSnapshots += DiscardedSnapshotsLastFrame;
        rxDiscardedSinceLastLog += DiscardedSnapshotsLastFrame;

        if (newestSnapshotJson != null)
            ProcessSnapshot(newestSnapshotJson);

        RenderPerformanceDisplay(now);
    }

    private void HandleReady(ReadyMessage ready)
    {
        if (ready == null || ready.bounds == null || ready.weights == null)
        {
            FaultConnection("boids-host sent an incomplete ready message.");
            return;
        }

        if (ready.protocolVersion != BoidsProtocol.Version)
        {
            FaultConnection(
                $"boids-host protocol version {ready.protocolVersion} does not match Unity version {BoidsProtocol.Version}.");
            return;
        }

        if (!BoundsAreValid(ready.bounds))
        {
            FaultConnection("boids-host sent invalid simulation bounds.");
            return;
        }

        LatestBounds = ready.bounds;
        LatestWeights = ready.weights;
        boundsView.Show(ready.bounds);
        ConnectionState = BoidsConnectionState.Ready;
    }

    private void HandleWeightsUpdated(WeightsUpdatedMessage message)
    {
        if (message == null || message.weights == null)
        {
            Debug.LogWarning("boids-host sent an incomplete weightsUpdated message.", this);
            return;
        }

        LatestWeights = message.weights;
    }

    private void HandleProtocolError(ProtocolErrorMessage error)
    {
        if (error == null)
        {
            Debug.LogWarning("boids-host sent an invalid error message.", this);
            return;
        }

        Debug.LogWarning($"boids-host protocol error [{error.code}]: {error.message}", this);

        if (error.code == "unsupportedProtocolVersion")
            FaultConnection(error.message);
    }

    private void ProcessSnapshot(string json)
    {
        if (ConnectionState != BoidsConnectionState.Ready)
        {
            Debug.LogWarning("Ignored a boids snapshot received before the ready handshake.", this);
            return;
        }

        try
        {
            WorldSnapshot snapshot = JsonUtility.FromJson<WorldSnapshot>(json);
            if (snapshot == null || snapshot.boids == null)
            {
                Debug.LogWarning($"Invalid boids snapshot: {json}", this);
                return;
            }

            AcceptSnapshot(snapshot);
        }
        catch (Exception exception)
        {
            Debug.LogWarning($"Unable to deserialise boids snapshot: {exception.Message}", this);
        }
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
        if (now < nextHealthLogAt || LatestHostHealth == null || performanceDisplay == null)
            return;

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
            Debug.Log($"boids-host: {message}", this);
    }

    private void CheckReadyTimeout(double now)
    {
        if (ConnectionState != BoidsConnectionState.AwaitingReady)
            return;

        if (!helloRetried && now >= helloRetryAt)
        {
            helloRetried = true;
            if (!SendCommand(new HelloCommand()))
                return;
        }

        if (now >= readyDeadline)
            FaultConnection($"boids-host did not become ready within {ReadyTimeoutSeconds:F0} seconds.");
    }

    private void CheckForUnexpectedHostExit()
    {
        Process process = hostProcess;
        if (process == null || ConnectionState == BoidsConnectionState.Stopped ||
            ConnectionState == BoidsConnectionState.Faulted)
            return;

        try
        {
            if (!process.HasExited)
                return;

            int exitCode = process.ExitCode;
            hostProcess = null;
            process.Dispose();
            FaultConnection($"boids-host exited unexpectedly with code {exitCode}.");
        }
        catch (Exception exception)
        {
            FaultConnection($"Unable to inspect boids-host process state: {exception.Message}");
        }
    }

    private bool SendCommand(object command)
    {
        Process process = hostProcess;
        if (process == null)
        {
            FaultConnection("Cannot send a command because boids-host is not running.");
            return false;
        }

        try
        {
            if (process.HasExited)
            {
                FaultConnection($"Cannot send a command because boids-host exited with code {process.ExitCode}.");
                return false;
            }

            string json = JsonUtility.ToJson(command);
            process.StandardInput.WriteLine(json);
            process.StandardInput.Flush();
            return true;
        }
        catch (Exception exception)
        {
            FaultConnection($"Unable to send command to boids-host: {exception.Message}");
            return false;
        }
    }

    private void FaultConnection(string message)
    {
        Debug.LogError(message, this);
        ConnectionState = BoidsConnectionState.Faulted;
        StopHost();
    }

    private static bool BoundsAreValid(BoundsSnapshot bounds)
    {
        return IsFinite(bounds.min.x) && IsFinite(bounds.min.y) && IsFinite(bounds.min.z) &&
               IsFinite(bounds.max.x) && IsFinite(bounds.max.y) && IsFinite(bounds.max.z) &&
               bounds.min.x < bounds.max.x &&
               bounds.min.y < bounds.max.y &&
               bounds.min.z < bounds.max.z;
    }

    private static bool IsFinite(float value)
    {
        return !float.IsNaN(value) && !float.IsInfinity(value);
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

    private GameObject CreateBoid(int boidId)
    {
        GameObject boid = Instantiate(boidPrefab, boidsRoot, false);
        boid.name = $"Boid {boidId}";
        return boid;
    }
}
