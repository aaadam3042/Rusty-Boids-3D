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
    
    private readonly ConcurrentQueue<string> snapshots = new();
    private readonly ConcurrentQueue<string> diagnostics = new();
    private readonly Dictionary<int, GameObject> boidsById = new();

    private Process hostProcess;
    private GameObject boidsEmpty;

    // Start is called once before the first execution of Update after the MonoBehaviour is created
    void Start()
    {
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
        while (snapshots.TryDequeue(out string json))
        {
            try
            {
                WorldSnapshot snapshot = JsonUtility.FromJson<WorldSnapshot>(json);

                if (snapshot == null || snapshot.boids == null)
                {
                    Debug.LogWarning($"Invalid boids snapshot: {json}");
                    continue;
                }

                ApplySnapshot(snapshot);
            } catch (Exception e)
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
