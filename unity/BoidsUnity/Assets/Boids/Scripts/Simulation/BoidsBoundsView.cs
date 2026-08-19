using UnityEngine;
using UnityEngine.Rendering;

[DisallowMultipleComponent]
public sealed class BoidsBoundsView : MonoBehaviour
{
    private static readonly int BaseColorProperty = Shader.PropertyToID("_BaseColor");
    private static readonly int ColorProperty = Shader.PropertyToID("_Color");

    [SerializeField] private Material boundaryMaterial;
    [SerializeField] private Color boundaryColor = new(0.2f, 0.8f, 1.0f, 1.0f);

    private GameObject boundsObject;
    private Mesh boundsMesh;
    private MeshRenderer boundsRenderer;
    private Material generatedMaterial;

    public void Show(BoundsSnapshot bounds)
    {
        if (bounds == null)
            return;

        if (!EnsureVisual())
            return;

        Vector3 min = bounds.min;
        Vector3 max = bounds.max;
        var vertices = new[]
        {
            new Vector3(min.x, min.y, min.z),
            new Vector3(max.x, min.y, min.z),
            new Vector3(max.x, max.y, min.z),
            new Vector3(min.x, max.y, min.z),
            new Vector3(min.x, min.y, max.z),
            new Vector3(max.x, min.y, max.z),
            new Vector3(max.x, max.y, max.z),
            new Vector3(min.x, max.y, max.z)
        };
        var indices = new[]
        {
            0, 1, 1, 2, 2, 3, 3, 0,
            4, 5, 5, 6, 6, 7, 7, 4,
            0, 4, 1, 5, 2, 6, 3, 7
        };

        boundsMesh.Clear();
        boundsMesh.vertices = vertices;
        boundsMesh.SetIndices(indices, MeshTopology.Lines, 0);
        boundsMesh.RecalculateBounds();

        var propertyBlock = new MaterialPropertyBlock();
        propertyBlock.SetColor(BaseColorProperty, boundaryColor);
        propertyBlock.SetColor(ColorProperty, boundaryColor);
        boundsRenderer.SetPropertyBlock(propertyBlock);
        boundsObject.SetActive(true);
    }

    private bool EnsureVisual()
    {
        if (boundsObject != null)
            return true;

        Material material = boundaryMaterial;
        if (material == null)
        {
            Shader shader = Shader.Find("Universal Render Pipeline/Unlit");
            if (shader == null)
                shader = Shader.Find("Unlit/Color");

            if (shader == null)
            {
                Debug.LogError("Unable to find an unlit shader for the simulation bounds.", this);
                return false;
            }

            generatedMaterial = new Material(shader)
            {
                name = "Generated Simulation Bounds Material"
            };
            material = generatedMaterial;
        }

        boundsObject = new GameObject("Simulation Bounds");
        boundsObject.transform.SetParent(transform, false);

        var meshFilter = boundsObject.AddComponent<MeshFilter>();
        boundsRenderer = boundsObject.AddComponent<MeshRenderer>();
        boundsRenderer.sharedMaterial = material;
        boundsRenderer.shadowCastingMode = ShadowCastingMode.Off;
        boundsRenderer.receiveShadows = false;

        boundsMesh = new Mesh { name = "Simulation Bounds Mesh" };
        meshFilter.sharedMesh = boundsMesh;
        return true;
    }

    private void OnDestroy()
    {
        if (boundsMesh != null)
            Destroy(boundsMesh);

        if (generatedMaterial != null)
            Destroy(generatedMaterial);
    }
}
