import AppKit
import Metal
import IOSurface
import QuartzCore

class MetalView: NSView {
    private var device: MTLDevice!
    private var commandQueue: MTLCommandQueue!
    private var metalLayer: CAMetalLayer!
    private var displayLink: CVDisplayLink?
    private var pipelineState: MTLRenderPipelineState?
    private var externalTexture: MTLTexture?

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setup()
    }

    override var wantsUpdateLayer: Bool { true }

    override func makeBackingLayer() -> CALayer {
        let layer = CAMetalLayer()
        layer.device = MTLCreateSystemDefaultDevice()
        layer.pixelFormat = .bgra8Unorm
        layer.framebufferOnly = true
        layer.contentsScale = window?.backingScaleFactor ?? 2.0
        return layer
    }

    private func setup() {
        wantsLayer = true
        layerContentsRedrawPolicy = .duringViewResize

        metalLayer = layer as? CAMetalLayer
        guard metalLayer != nil else {
            fatalError("Failed to create CAMetalLayer")
        }

        device = metalLayer.device
        guard device != nil else {
            fatalError("No Metal device available")
        }

        commandQueue = device.makeCommandQueue()

        setupPipeline()
        startDisplayLink()
    }

    private func setupPipeline() {
        let shaderSource = """
        #include <metal_stdlib>
        using namespace metal;

        struct VertexOut {
            float4 position [[position]];
            float2 texcoord;
        };

        vertex VertexOut vertex_fullscreen(uint vid [[vertex_id]]) {
            // Full-screen triangle: 3 vertices cover the entire viewport
            float2 positions[3] = {
                float2(-1.0, -1.0),
                float2( 3.0, -1.0),
                float2(-1.0,  3.0)
            };
            // Map clip space to texture coordinates (flip Y for top-left origin)
            float2 texcoords[3] = {
                float2(0.0, 1.0),
                float2(2.0, 1.0),
                float2(0.0, -1.0)
            };
            VertexOut out;
            out.position = float4(positions[vid], 0.0, 1.0);
            out.texcoord = texcoords[vid];
            return out;
        }

        fragment float4 fragment_textured(VertexOut in [[stage_in]],
                                          texture2d<float> tex [[texture(0)]]) {
            constexpr sampler s(mag_filter::linear, min_filter::linear);
            return tex.sample(s, in.texcoord);
        }
        """

        do {
            let library = try device.makeLibrary(source: shaderSource, options: nil)
            let vertexFunc = library.makeFunction(name: "vertex_fullscreen")
            let fragmentFunc = library.makeFunction(name: "fragment_textured")

            let desc = MTLRenderPipelineDescriptor()
            desc.vertexFunction = vertexFunc
            desc.fragmentFunction = fragmentFunc
            desc.colorAttachments[0].pixelFormat = metalLayer.pixelFormat

            pipelineState = try device.makeRenderPipelineState(descriptor: desc)
            NSLog("[MetalView] Shader pipeline created")
        } catch {
            NSLog("[MetalView] Failed to create pipeline: %@", error.localizedDescription)
        }
    }

    /// Accept an external IOSurface and create a Metal texture backed by it (zero-copy).
    func setExternalSurface(_ surface: IOSurface) {
        let width = IOSurfaceGetWidth(surface)
        let height = IOSurfaceGetHeight(surface)

        let descriptor = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .bgra8Unorm,
            width: width,
            height: height,
            mipmapped: false
        )
        descriptor.usage = [.shaderRead]
        descriptor.storageMode = .shared

        let texture = device.makeTexture(
            descriptor: descriptor,
            iosurface: surface,
            plane: 0
        )

        if let texture = texture {
            externalTexture = texture
            NSLog("[MetalView] Created texture from IOSurface: %dx%d", width, height)
        } else {
            NSLog("[MetalView] Failed to create texture from IOSurface")
        }
    }

    override func viewDidChangeBackingProperties() {
        super.viewDidChangeBackingProperties()
        metalLayer?.contentsScale = window?.backingScaleFactor ?? 2.0
    }

    override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        let scale = window?.backingScaleFactor ?? 2.0
        metalLayer?.drawableSize = CGSize(
            width: newSize.width * scale,
            height: newSize.height * scale
        )
    }

    private func startDisplayLink() {
        CVDisplayLinkCreateWithActiveCGDisplays(&displayLink)
        guard let displayLink = displayLink else { return }

        let callback: CVDisplayLinkOutputCallback = {
            (_, _, _, _, _, userInfo) -> CVReturn in
            let view = Unmanaged<MetalView>.fromOpaque(userInfo!).takeUnretainedValue()
            DispatchQueue.main.async { view.render() }
            return kCVReturnSuccess
        }

        CVDisplayLinkSetOutputCallback(
            displayLink,
            callback,
            Unmanaged.passUnretained(self).toOpaque()
        )
        CVDisplayLinkStart(displayLink)
    }

    private func render() {
        guard let drawable = metalLayer.nextDrawable() else { return }

        let passDescriptor = MTLRenderPassDescriptor()
        passDescriptor.colorAttachments[0].texture = drawable.texture
        passDescriptor.colorAttachments[0].loadAction = .clear
        passDescriptor.colorAttachments[0].storeAction = .store
        passDescriptor.colorAttachments[0].clearColor = MTLClearColor(
            red: 0.15, green: 0.15, blue: 0.15, alpha: 1.0
        )

        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: passDescriptor)
        else { return }

        // If we have an external texture and pipeline, render it
        if let pipeline = pipelineState, let texture = externalTexture {
            encoder.setRenderPipelineState(pipeline)
            encoder.setFragmentTexture(texture, index: 0)
            encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)
        }

        encoder.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    deinit {
        if let displayLink = displayLink {
            CVDisplayLinkStop(displayLink)
        }
    }
}
