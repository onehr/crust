int a[100];

int EXIT_FAILURE = 1;
int EXIT_SUCCESS = 0;

int main() {
        int tmp;

        for (int i = 0; i < 100; i = i + 1) {
                a[i] = 99 - i;
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (a[i] != 99 - i) return EXIT_FAILURE;
        }
        int len = 100;
        for (int i = 0; i < len - 1; i = i + 1) {
                for (int j = 0; j < len - 1 - i; j = j + 1)
                        if (a[j] > a[j + 1]) {
                                tmp = a[j];
                                a[j] = a[j + 1];
                                a[j + 1] = tmp;
                        }
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (a[i] != i) return EXIT_FAILURE;
        }

        return a[10];
}
