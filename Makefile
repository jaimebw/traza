PREFIX ?= ~/.local
BINDIR = $(PREFIX)/bin

install:
	mkdir -p $(BINDIR)
	cp target/release/traza $(BINDIR)/traza
	chmod 755 $(BINDIR)/traza

uninstall:
	rm -f $(BINDIR)/traza

