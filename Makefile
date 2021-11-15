PROG = test

%: %.bf
	target/debug/brainfuck_rs $^ -o $@.o
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