CC = gcc
CFLAGS = -Wall -Wextra -O2 -std=c99
TARGET = yan
SRC = src/yan.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

clean:
	rm -f $(TARGET)

test: $(TARGET)
	./$(TARGET)

.PHONY: all clean test
