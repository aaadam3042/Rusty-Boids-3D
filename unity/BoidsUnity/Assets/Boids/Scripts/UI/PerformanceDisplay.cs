using TMPro;
using UnityEngine;

public sealed class PerformanceDisplay : MonoBehaviour
{
    private const double SimSpeedWarningThreshold = 0.98;
    private const double SimSpeedCriticalThreshold = 0.90;
    private const double HostLateWarningMs = 16.67;
    private const double HostLateCriticalMs = 100.0;
    private const double TickBudgetWarningRatio = 0.80;
    private const long DiscardedWarningThreshold = 1;
    private const long DiscardedCriticalThreshold = 6;
    private const double FpsWarningThreshold = 50.0;
    private const double FpsCriticalThreshold = 30.0;

    [SerializeField] private TMP_Text simSpeedValue;
    [SerializeField] private TMP_Text hostLateValue;
    [SerializeField] private TMP_Text stepValue;
    [SerializeField] private TMP_Text publishValue;
    [SerializeField] private TMP_Text discardedValue;
    [SerializeField] private TMP_Text fpsValue;

    [Header("Status Colours")]
    [SerializeField] private Color normalColour = Color.white;
    [SerializeField] private Color warningColour = new Color32(255, 165, 0, 255);
    [SerializeField] private Color criticalColour = Color.red;

    public void Render(
        HostHealthSnapshot health,
        long discardedSnapshots,
        long totalDiscardedSnapshots,
        double unityFps)
    {
        if (health == null)
            return;

        simSpeedValue.text = health.realTimeFactorReady
            ? $"{health.realTimeFactor:F2}×"
            : "—";

        hostLateValue.text = $"{health.deadlineLatenessMs:F2} ms";
        stepValue.text = $"{health.lastStepMs:F2} ms";
        publishValue.text = $"{health.previousPublishMs:F2} ms";
        discardedValue.text = $"{discardedSnapshots} ({totalDiscardedSnapshots} total)";
        fpsValue.text = $"{unityFps:F1}";

        simSpeedValue.color = health.realTimeFactorReady
            ? ColourForLowValue(
                health.realTimeFactor,
                SimSpeedWarningThreshold,
                SimSpeedCriticalThreshold)
            : normalColour;

        hostLateValue.color = ColourForHighValue(
            health.deadlineLatenessMs,
            HostLateWarningMs,
            HostLateCriticalMs);

        double tickBudgetMs = health.fixedDtSeconds * 1_000.0;
        double tickBudgetWarningMs = tickBudgetMs * TickBudgetWarningRatio;

        stepValue.color = ColourForHighValue(
            health.lastStepMs,
            tickBudgetWarningMs,
            tickBudgetMs);

        publishValue.color = ColourForHighValue(
            health.previousPublishMs,
            tickBudgetWarningMs,
            tickBudgetMs);

        discardedValue.color = ColourForHighValue(
            discardedSnapshots,
            DiscardedWarningThreshold,
            DiscardedCriticalThreshold);

        fpsValue.color = ColourForLowValue(
            unityFps,
            FpsWarningThreshold,
            FpsCriticalThreshold);
    }

    private Color ColourForLowValue(double value, double warningBelow, double criticalBelow)
    {
        if (value < criticalBelow)
            return criticalColour;

        if (value < warningBelow)
            return warningColour;

        return normalColour;
    }

    private Color ColourForHighValue(double value, double warningAt, double criticalAt)
    {
        if (value >= criticalAt)
            return criticalColour;

        if (value >= warningAt)
            return warningColour;

        return normalColour;
    }
}
