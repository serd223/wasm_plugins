// plug name is inferred from the file name as plug5
const char* __deps() {
    return "plug1";
}

extern int add(int a, int b);
extern void print(int n);

void hello_from_c(int a, int b) {
    print(add(a, b));
}
