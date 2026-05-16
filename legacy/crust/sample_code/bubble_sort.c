int a[15];

int main(void) {
        int tmp;

        for (int i = 0; i < 15; i = i + 1) {
                a[i] = 15 - i;
        }

        printf("Before bubble sort:\n");
        for (int i = 0; i < 15; i = i + 1) {
                printf("%d ", a[i]);
        }
        printf("\n");

        int len = 15;
        for (int i = 0; i < len - 1; i = i + 1) {
                for (int j = 0; j < len - 1 - i; j = j + 1)
                        if (a[j] > a[j + 1]) {
                                tmp = a[j];
                                a[j] = a[j + 1];
                                a[j + 1] = tmp;
                        }
        }

        printf("After bubble sort:\n");
        for (int i = 0; i < 15; i = i + 1) {
                printf("%d ", a[i]);
        }
        printf("\n");

        return 0;
}
