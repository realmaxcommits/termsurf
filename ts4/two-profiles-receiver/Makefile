all: receiver

receiver: main.m
	clang -framework Foundation -framework IOSurface -o receiver main.m

clean:
	rm -f receiver

.PHONY: all clean
