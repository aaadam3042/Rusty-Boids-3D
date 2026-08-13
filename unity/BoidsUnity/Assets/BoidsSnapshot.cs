using System;
using UnityEngine;

[Serializable]
public sealed class WorldSnapshot
{
    public long tick;
    public BoidSnapshot[] boids;
    public HostHealthSnapshot health;
}

[Serializable]
public sealed class HostHealthSnapshot
{
    public double simulationTimeSeconds;
    public double fixedDtSeconds;
    public double realTimeFactor;
    public bool realTimeFactorReady;
    public double deadlineLatenessMs;
    public double lastStepMs;
    public double previousPublishMs;
}

[Serializable]
public sealed class BoidSnapshot
{
    public int id;
    public Vector3 position;
    public Vector3 velocity;
}