using System;
using UnityEngine;

public static class BoidsProtocol
{
    public const int Version = 1;
}

public enum BoidsConnectionState
{
    Stopped,
    Starting,
    AwaitingReady,
    Ready,
    Faulted
}

[Serializable]
public sealed class HostMessageHeader
{
    public string type;
}

[Serializable]
public sealed class ReadyMessage
{
    public string type;
    public int protocolVersion;
    public BoundsSnapshot bounds;
    public WeightsSnapshot weights;
}

[Serializable]
public sealed class BoundsSnapshot
{
    public Vector3 min;
    public Vector3 max;
}

[Serializable]
public sealed class WeightsSnapshot
{
    public float cohesion;
    public float alignment;
    public float separation;
}

[Serializable]
public sealed class WeightsUpdatedMessage
{
    public string type;
    public WeightsSnapshot weights;
}

[Serializable]
public sealed class ProtocolErrorMessage
{
    public string type;
    public string code;
    public string message;
}

[Serializable]
public sealed class HelloCommand
{
    public string type = "hello";
    public int protocolVersion = BoidsProtocol.Version;
}

[Serializable]
public sealed class SetWeightsCommand
{
    public string type = "setWeights";
    public WeightsSnapshot weights;

    public SetWeightsCommand(float cohesion, float alignment, float separation)
    {
        weights = new WeightsSnapshot
        {
            cohesion = cohesion,
            alignment = alignment,
            separation = separation
        };
    }
}

[Serializable]
public sealed class ShutdownCommand
{
    public string type = "shutdown";
}
