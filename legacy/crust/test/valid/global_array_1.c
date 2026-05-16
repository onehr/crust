
int a[100];

int main() {
        for (int i = 0; i < 100; i = i + 1) {
                for (int j = 0; j < 100; j = j + 1) {
                        a[i] = a[i] + j;
                }
        }
        return a[10];
}
