const char* deps() {
    return "plug1";
}

extern int add(int a, int b);
extern void print(int n);

void hello_from_c(int a, int b) {
    print(add(a, b));
}
