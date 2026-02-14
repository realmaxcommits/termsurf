all: receiver shaders.metallib

receiver: main.mm
	clang++ -std=c++17 -fobjc-arc \
	  -framework Foundation -framework IOSurface \
	  -framework Metal -framework QuartzCore -framework AppKit \
	  -framework CoreVideo -o receiver main.mm

shaders.metallib: shaders.metal
	xcrun -sdk macosx metal -c shaders.metal -o shaders.air
	xcrun -sdk macosx metallib shaders.air -o shaders.metallib
	rm -f shaders.air

clean:
	rm -f receiver shaders.metallib shaders.air

.PHONY: all clean
