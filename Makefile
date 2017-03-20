LDLIBS=-lpthread -lutil

main: main.o

clean:
	rm -rf main main.o
