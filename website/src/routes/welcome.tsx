import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { TextGeometry } from "three/addons/geometries/TextGeometry.js";
import { FontLoader } from "three/addons/loaders/FontLoader.js";

export const Route = createFileRoute("/welcome")({
  component: WelcomePage,
});

function WelcomePage() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [showModal, setShowModal] = useState(false);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Scene setup
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a0a0a);

    const camera = new THREE.PerspectiveCamera(
      50,
      container.clientWidth / container.clientHeight,
      0.1,
      1000,
    );
    // Pull camera back on narrow viewports so text always fits.
    const baseZ = 12;
    const baseAspect = 16 / 9;
    function getCameraZ() {
      const aspect = container!.clientWidth / container!.clientHeight;
      return aspect < baseAspect ? baseZ * (baseAspect / aspect) : baseZ;
    }
    camera.position.z = getCameraZ();

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(container.clientWidth, container.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    container.appendChild(renderer.domElement);

    // Lights
    const ambient = new THREE.AmbientLight(0x808080, 1.5);
    scene.add(ambient);

    const light1 = new THREE.PointLight(0x00ffff, 80, 50);
    light1.position.set(5, 5, 8);
    scene.add(light1);

    const light2 = new THREE.PointLight(0xff00ff, 80, 50);
    light2.position.set(-5, -3, 8);
    scene.add(light2);

    const light3 = new THREE.PointLight(0xffff00, 40, 50);
    light3.position.set(0, 8, 5);
    scene.add(light3);

    // Load font and create text
    const loader = new FontLoader();
    let internetMesh: THREE.Mesh | null = null;

    loader.load(
      "https://cdn.jsdelivr.net/npm/three@0.183.2/examples/fonts/helvetiker_bold.typeface.json",
      (font) => {
        // "welcome to" — flat, above
        const welcomeGeo = new TextGeometry("welcome to", {
          font,
          size: 0.4,
          depth: 0.05,
          curveSegments: 12,
        });
        welcomeGeo.computeBoundingBox();
        welcomeGeo.center();

        const welcomeMat = new THREE.MeshStandardMaterial({
          color: 0x888888,
          metalness: 0.3,
          roughness: 0.7,
        });
        const welcomeMesh = new THREE.Mesh(welcomeGeo, welcomeMat);
        welcomeMesh.position.y = 2.5;
        scene.add(welcomeMesh);

        // "THE INTERNET" — big, 3D, spinning
        const internetGeo = new TextGeometry("THE INTERNET", {
          font,
          size: 1.2,
          depth: 0.6,
          curveSegments: 16,
          bevelEnabled: true,
          bevelThickness: 0.08,
          bevelSize: 0.04,
          bevelSegments: 8,
        });
        internetGeo.computeBoundingBox();
        internetGeo.center();

        const internetMat = new THREE.MeshStandardMaterial({
          color: 0x00e5ff,
          metalness: 0.9,
          roughness: 0.1,
          envMapIntensity: 1.0,
        });
        internetMesh = new THREE.Mesh(internetGeo, internetMat);
        internetMesh.position.y = 0;
        scene.add(internetMesh);

        // Subtitle — small, below
        const subtitleGeo = new TextGeometry(
          "advanced communications network for planet earth",
          {
            font,
            size: 0.22,
            depth: 0.02,
            curveSegments: 12,
          },
        );
        subtitleGeo.computeBoundingBox();
        subtitleGeo.center();

        const subtitleMat = new THREE.MeshStandardMaterial({
          color: 0xaaaaaa,
          metalness: 0.3,
          roughness: 0.7,
        });
        const subtitleMesh = new THREE.Mesh(subtitleGeo, subtitleMat);
        subtitleMesh.position.y = -2;
        scene.add(subtitleMesh);
      },
    );

    // Stars
    const starsGeo = new THREE.BufferGeometry();
    const starPositions = new Float32Array(3000);
    for (let i = 0; i < 3000; i++) {
      starPositions[i] = (Math.random() - 0.5) * 100;
    }
    starsGeo.setAttribute(
      "position",
      new THREE.BufferAttribute(starPositions, 3),
    );
    const starsMat = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 0.05,
    });
    const stars = new THREE.Points(starsGeo, starsMat);
    scene.add(stars);

    // Animation
    let animationId: number;
    const clock = new THREE.Clock();

    function animate() {
      animationId = requestAnimationFrame(animate);
      const t = clock.getElapsedTime();

      if (internetMesh) {
        internetMesh.rotation.y = Math.sin(t * 0.5) * 0.4;
        internetMesh.rotation.x = Math.sin(t * 0.3) * 0.1;
        internetMesh.position.y = Math.sin(t * 0.8) * 0.3;
      }

      // Slowly rotate lights
      light1.position.x = Math.sin(t * 0.7) * 8;
      light1.position.y = Math.cos(t * 0.5) * 5;
      light2.position.x = Math.cos(t * 0.3) * 8;
      light2.position.y = Math.sin(t * 0.4) * 5;

      stars.rotation.y = t * 0.02;
      stars.rotation.x = t * 0.01;

      renderer.render(scene, camera);
    }
    animate();

    // Resize handler
    function onResize() {
      if (!container) return;
      camera.aspect = container.clientWidth / container.clientHeight;
      camera.position.z = getCameraZ();
      camera.updateProjectionMatrix();
      renderer.setSize(container.clientWidth, container.clientHeight);
    }
    window.addEventListener("resize", onResize);

    return () => {
      cancelAnimationFrame(animationId);
      window.removeEventListener("resize", onResize);
      renderer.dispose();
      container.removeChild(renderer.domElement);
    };
  }, []);

  return (
    <div
      style={{
        position: "relative",
        width: "100vw",
        height: "100vh",
        overflow: "hidden",
      }}
    >
      <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
      <button
        onClick={() => setShowModal(true)}
        style={{
          position: "absolute",
          bottom: "12%",
          left: "50%",
          transform: "translateX(-50%)",
          padding: "14px 40px",
          fontSize: "16px",
          fontFamily: "Helvetica, Arial, sans-serif",
          letterSpacing: "2px",
          textTransform: "uppercase",
          color: "#fff",
          background: "rgba(0, 229, 255, 0.15)",
          border: "1px solid rgba(0, 229, 255, 0.5)",
          borderRadius: "4px",
          cursor: "pointer",
          transition: "all 0.3s ease",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "rgba(0, 229, 255, 0.3)";
          e.currentTarget.style.borderColor = "rgba(0, 229, 255, 0.8)";
          e.currentTarget.style.boxShadow = "0 0 20px rgba(0, 229, 255, 0.3)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "rgba(0, 229, 255, 0.15)";
          e.currentTarget.style.borderColor = "rgba(0, 229, 255, 0.5)";
          e.currentTarget.style.boxShadow = "none";
        }}
      >
        Begin Your Journey
      </button>

      {showModal && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "rgba(0, 0, 0, 0.85)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 10,
            fontFamily: "Helvetica, Arial, sans-serif",
          }}
          onClick={() => setShowModal(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: "24px",
              maxWidth: "640px",
              width: "90%",
            }}
          >
            <iframe
              width="560"
              height="315"
              src="https://www.youtube.com/embed/dQw4w9WgXcQ?autoplay=1"
              title="Welcome to the Internet"
              frameBorder="0"
              allow="autoplay; encrypted-media"
              allowFullScreen
              style={{
                borderRadius: "8px",
                border: "1px solid rgba(0, 229, 255, 0.3)",
                maxWidth: "100%",
              }}
            />

            <p
              style={{
                color: "#ccc",
                fontSize: "18px",
                textAlign: "center",
                lineHeight: 1.6,
                margin: 0,
              }}
            >
              You have been Rick Rolled.
              <br />
              Everybody gets Rick Rolled.
              <br />
              <span style={{ color: "#00e5ff" }}>Welcome to the Internet.</span>
            </p>

            <div
              style={{
                display: "flex",
                gap: "16px",
                flexWrap: "wrap",
                justifyContent: "center",
              }}
            >
              <a
                href="https://github.com/termsurf/termsurf"
                style={linkStyle}
                onMouseEnter={linkHover}
                onMouseLeave={linkUnhover}
              >
                GitHub
              </a>
              <a
                href="https://termsurf.com"
                style={linkStyle}
                onMouseEnter={linkHover}
                onMouseLeave={linkUnhover}
              >
                termsurf.com
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

const linkStyle: React.CSSProperties = {
  color: "#00e5ff",
  textDecoration: "none",
  padding: "8px 20px",
  border: "1px solid rgba(0, 229, 255, 0.3)",
  borderRadius: "4px",
  fontSize: "14px",
  fontFamily: "Helvetica, Arial, sans-serif",
  letterSpacing: "1px",
  transition: "all 0.3s ease",
};

const linkHover = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.background = "rgba(0, 229, 255, 0.15)";
  e.currentTarget.style.borderColor = "rgba(0, 229, 255, 0.6)";
};

const linkUnhover = (e: React.MouseEvent<HTMLAnchorElement>) => {
  e.currentTarget.style.background = "transparent";
  e.currentTarget.style.borderColor = "rgba(0, 229, 255, 0.3)";
};
