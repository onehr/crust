int a[100];
int main(void) {
        int n  = 10;
        printf("Input array len: ");
        scanf("%lld", &n);
        printf("Now input %d numbers:\n", n);
        for (int i = 0; i < n; i = i + 1) {
                scanf("%d", &a[i]);
        }
        printf("Your input array:\n");
        for (int i = 0; i < n; i = i + 1) {
                printf("%d ", a[i]);
        }
        printf("\n");
        return 0;
}
