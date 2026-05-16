int array[100];

int main(void) {
        int len;
        scanf("%d", &len);

        for (int i = 0; i < len; i = i + 1) {
                scanf("%d", &array[i]);
        }

        printf("your input array:\n");
        for (int i = 0; i < len; i = i + 1) {
                printf("%d ", array[i]);
        }
        printf("\n");

        int tmp;
        for (int i = 0; i < len - 1; i = i + 1) {
                for (int j = 0; j < len - 1 - i; j = j + 1)
                        if (array[j] > array[j + 1]) {
                                tmp = array[j];
                                array[j] = array[j + 1];
                                array[j + 1] = tmp;
                        }
        }

        printf("After sort:\n");
        for (int i = 0; i < len; i = i + 1) {
                printf("%d ", array[i]);
        }
        printf("\n");
        return 0;
}
