int a[1000];
int b[1000];

int COMPILER_OK = 0;
int COMPILER_ERROR = 1;
int main() {
        for (int i = 0; i < 100; i = i + 1) {
                a[i] = i;
        }
        for (int i = 0; i < 100; i = i + 1) {
                b[i] = 99 - i;
        }
        int sum_a = 0;
        int sum_b = 0;
        for (int i = 0; i < 100; i = i + 1) {
                sum_a = sum_a + a[i];
        }
        for (int i = 0; i < 100; i = i + 1) {
                sum_b = sum_b + b[i];
        }

        if (sum_a == sum_b) return COMPILER_OK;
        else return COMPILER_ERROR;
}
