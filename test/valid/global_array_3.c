int a[100];
int b[100];

int main() {
        for (int i = 0; i < 100; i = i + 1) {
                a[i] = 1;
                b[i] = 0;
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (a[i] != 1) return 1;
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (b[i] != 0) return 1;
        }

        int tmp = 10;
        for (int i = 0; i < 100; i = i + 1) {
                tmp = a[i];
                a[i] = b[i];
                b[i] = tmp;
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (a[i] != 0) return 1;
        }
        for (int i = 0; i < 100; i = i + 1) {
                if (b[i] != 1) return 1;
        }

        for (int i = 0; i < 100; i = i + 1) {
                a[i] = b[i];
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (a[i] != 1) return 1;
        }
        for (int i = 0; i < 100; i = i + 1) {
                if (b[i] != 1) return 1;
        }
        return 0;
}
