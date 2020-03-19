void a123(int* p) {
    *p=1;
}

int _123(int *p, int q) {
    return *p+q;
}

int _SUM_(int p, int q) {
    return p+q;
}

int main() {
    int r = 0;
    a123(&r);
    r=_123(&r,1);
    r=_SUM_(r,2);
    printf("%d\n", r);
    return r;
}