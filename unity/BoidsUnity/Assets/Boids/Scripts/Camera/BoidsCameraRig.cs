using UnityEngine;
using UnityEngine.InputSystem;

[DisallowMultipleComponent]
[AddComponentMenu("Boids/Boids Camera Rig")]
public sealed class BoidsCameraRig : MonoBehaviour
{
    private const string CameraActionMapName = "Camera";

    [Header("References")]
    [SerializeField] private Camera controlledCamera;
    [SerializeField] private InputActionAsset inputActions;

    [Header("Initial View")]
    [SerializeField] private Vector3 initialFocus = new(50f, 50f, 50f);
    [SerializeField] private float initialDistance = 150f;
    [SerializeField] private float initialYaw;
    [SerializeField] private float initialPitch;

    [Header("Orbit")]
    [SerializeField] private float orbitSensitivity = 0.15f;
    [SerializeField] private float minimumPitch = -89f;
    [SerializeField] private float maximumPitch = 89f;

    [Header("Zoom")]
    [SerializeField] private float zoomSensitivity = 0.0015f;
    [SerializeField] private float minimumDistance = 2f;
    [SerializeField] private float maximumDistance = 500f;

    [Header("Free Movement")]
    [SerializeField] private float panSensitivity = 1f;
    [SerializeField] private float moveSpeed = 50f;
    [SerializeField] private float boostMultiplier = 4f;

    [Header("Tracking")]
    [SerializeField] private float trackingSmoothTime = 0.08f;

    private InputActionMap cameraActionMap;
    private InputAction pointerDeltaAction;
    private InputAction orbitAction;
    private InputAction panAction;
    private InputAction zoomAction;
    private InputAction moveAction;
    private InputAction elevateAction;
    private InputAction boostAction;
    private InputAction resetViewAction;
    private InputAction cancelTrackingAction;

    private Vector3 focusPoint;
    private Vector3 trackingVelocity;
    private Transform trackingTarget;
    private float yaw;
    private float pitch;
    private float distance;

    public bool IsTracking => trackingTarget != null;
    public Transform TrackingTarget => trackingTarget;
    public Vector3 FocusPoint => focusPoint;
    public float Distance => distance;

    private void Awake()
    {
        if (!TryInitialise())
        {
            enabled = false;
            return;
        }

        ResetView();
    }

    private void OnEnable()
    {
        cameraActionMap?.Enable();
    }

    private void OnDisable()
    {
        cameraActionMap?.Disable();
    }

    private void LateUpdate()
    {
        float deltaTime = Time.unscaledDeltaTime;
        Vector2 pointerDelta = pointerDeltaAction.ReadValue<Vector2>();

        if (resetViewAction.WasPressedThisFrame())
            ResetView();

        if (cancelTrackingAction.WasPressedThisFrame())
            StopTracking();

        UpdateTracking(deltaTime);

        if (panAction.IsPressed())
            UpdatePan(pointerDelta);
        else if (orbitAction.IsPressed())
            UpdateOrbit(pointerDelta);

        UpdateFreeMovement(deltaTime);
        UpdateZoom();
        ApplyPose();
    }

    public void Track(Transform target)
    {
        trackingTarget = target;
        trackingVelocity = Vector3.zero;
    }

    public void StopTracking()
    {
        trackingTarget = null;
        trackingVelocity = Vector3.zero;
    }

    public void Focus(Vector3 position)
    {
        StopTracking();
        focusPoint = position;
    }

    public void SetDistance(float newDistance)
    {
        distance = Mathf.Clamp(newDistance, minimumDistance, maximumDistance);
    }

    public void ResetView()
    {
        StopTracking();
        focusPoint = initialFocus;
        distance = Mathf.Clamp(initialDistance, minimumDistance, maximumDistance);
        yaw = initialYaw;
        pitch = Mathf.Clamp(initialPitch, minimumPitch, maximumPitch);
        ApplyPose();
    }

    private bool TryInitialise()
    {
        if (controlledCamera == null)
        {
            Debug.LogError("BoidsCameraRig requires a controlled Camera.", this);
            return false;
        }

        if (controlledCamera.transform.parent != transform)
        {
            Debug.LogError("The controlled Camera must be a direct child of the camera rig.", this);
            return false;
        }

        if (inputActions == null)
        {
            Debug.LogError("BoidsCameraRig requires a BoidsInputActions asset.", this);
            return false;
        }

        cameraActionMap = inputActions.FindActionMap(CameraActionMapName, true);
        pointerDeltaAction = cameraActionMap.FindAction("PointerDelta", true);
        orbitAction = cameraActionMap.FindAction("Orbit", true);
        panAction = cameraActionMap.FindAction("Pan", true);
        zoomAction = cameraActionMap.FindAction("Zoom", true);
        moveAction = cameraActionMap.FindAction("Move", true);
        elevateAction = cameraActionMap.FindAction("Elevate", true);
        boostAction = cameraActionMap.FindAction("Boost", true);
        resetViewAction = cameraActionMap.FindAction("ResetView", true);
        cancelTrackingAction = cameraActionMap.FindAction("CancelTracking", true);
        return true;
    }

    private void UpdateTracking(float deltaTime)
    {
        if (trackingTarget == null)
            return;

        focusPoint = Vector3.SmoothDamp(
            focusPoint,
            trackingTarget.position,
            ref trackingVelocity,
            trackingSmoothTime,
            Mathf.Infinity,
            deltaTime);
    }

    private void UpdateOrbit(Vector2 pointerDelta)
    {
        yaw += pointerDelta.x * orbitSensitivity;
        pitch -= pointerDelta.y * orbitSensitivity;
        pitch = Mathf.Clamp(pitch, minimumPitch, maximumPitch);
    }

    private void UpdatePan(Vector2 pointerDelta)
    {
        if (pointerDelta.sqrMagnitude == 0f)
            return;

        StopTracking();

        Quaternion rotation = Quaternion.Euler(pitch, yaw, 0f);
        float worldUnitsPerPixel =
            2f * distance *
            Mathf.Tan(controlledCamera.fieldOfView * 0.5f * Mathf.Deg2Rad) /
            Mathf.Max(Screen.height, 1);

        Vector3 right = rotation * Vector3.right;
        Vector3 up = rotation * Vector3.up;

        focusPoint -=
            (right * pointerDelta.x + up * pointerDelta.y) *
            worldUnitsPerPixel *
            panSensitivity;
    }

    private void UpdateFreeMovement(float deltaTime)
    {
        Vector2 movement = moveAction.ReadValue<Vector2>();
        float elevation = elevateAction.ReadValue<float>();
        Vector3 localDirection = new(movement.x, elevation, movement.y);

        if (localDirection.sqrMagnitude == 0f)
            return;

        StopTracking();

        if (localDirection.sqrMagnitude > 1f)
            localDirection.Normalize();

        Quaternion rotation = Quaternion.Euler(pitch, yaw, 0f);
        float currentMoveSpeed = moveSpeed;

        if (boostAction.IsPressed())
            currentMoveSpeed *= boostMultiplier;

        focusPoint += rotation * localDirection * currentMoveSpeed * deltaTime;
    }

    private void UpdateZoom()
    {
        float scroll = zoomAction.ReadValue<float>();

        if (Mathf.Approximately(scroll, 0f))
            return;

        distance *= Mathf.Exp(-scroll * zoomSensitivity);
        distance = Mathf.Clamp(distance, minimumDistance, maximumDistance);
    }

    private void ApplyPose()
    {
        transform.SetPositionAndRotation(
            focusPoint,
            Quaternion.Euler(pitch, yaw, 0f));

        controlledCamera.transform.localPosition = Vector3.back * distance;
        controlledCamera.transform.localRotation = Quaternion.identity;
    }

    private void OnValidate()
    {
        minimumDistance = Mathf.Max(0.01f, minimumDistance);
        maximumDistance = Mathf.Max(minimumDistance, maximumDistance);
        initialDistance = Mathf.Clamp(initialDistance, minimumDistance, maximumDistance);
        maximumPitch = Mathf.Max(minimumPitch, maximumPitch);
        trackingSmoothTime = Mathf.Max(0f, trackingSmoothTime);
    }
}
