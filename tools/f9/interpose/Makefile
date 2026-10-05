CC ?= gcc
CFLAGS ?= -std=c11 -O2 -Wall -Wextra -Wpedantic -Werror -fPIC
LDFLAGS ?= -shared
TARGET = libf9-getenv-observer.so

all: $(TARGET)

$(TARGET): getenv_observer.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<

clean:
	rm -f $(TARGET)

.PHONY: all clean
