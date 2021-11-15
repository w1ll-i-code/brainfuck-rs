PROG = test

%: %.bf
	cargo run -- $^ -o $@.o -O Max
	gcc $@.o -o $@
	chmod 777 $@

.PHONY: all
all: $(PROG)

.PHONY: clean
clean:
	$(RM) $(PROG) $(PROG).o

.PHONY: run
run: $(PROG)
	./$(PROG)