using System;
using UnityEngine;

[Serializable]
public sealed class WorldSnapshot
{
    public long tick;
    public BoidSnapshot[] boids;
}

[Serializable]
public sealed class BoidSnapshot
{
    public int id;
    public Vector3 position;
    public Vector3 velocity;
}