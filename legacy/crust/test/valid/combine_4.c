int fib(int a) {if (a == 0 || a == 1) {return a;} else {return fib(a - 1) + fib(a - 2);}}
int max(int a, int b) {return a > b ? a : b;}
int min(int a, int b) {return a < b ? a : b;}
int sum(int a, int b) {return a + b;}
int mul(int a, int b) {return a * b;}
int div(int a, int b) {return a / b;}

int EXIT_SUCCESS = 0;
int EXIT_FAILURE = 1;

int arr[100];

int main() {
        int a = 2;
        int b = 3;
        int n = 10;
        int len = 100;

        for (int i = 0; i < 30; i = i + 1) {
                if (i == 0 || i == 1) arr[i] = i;
                else arr[i] = arr[i-1] + arr[i-2];
        }

        for (int i = 0; i < 30; i = i + 1) {
                if (arr[i] != fib(i)) return EXIT_FAILURE;
        }

        for (int i = 0; i < len; i = i + 1) {
                arr[i] = len - 1 - i;
        }

        for (int i = 0; i < len; i = i + 1) {
                if (arr[i] != len - 1 - i) return EXIT_FAILURE;
        }

        int tmp = 0;
        for (int i = 0; i < len - 1; i = i + 1) {
                for (int j = 0; j < len - 1 - i; j = j + 1)
                        if (arr[j] > arr[j + 1]) {
                                tmp = arr[j];
                                arr[j] = arr[j + 1];
                                arr[j + 1] = tmp;
                        }
        }

        for (int i = 0; i < 100; i = i + 1) {
                if (arr[i] != i) return EXIT_FAILURE;
        }


        if (fib(n) != 55) return EXIT_FAILURE;

        if (min(a, b) != 2) return EXIT_FAILURE;

        if (max(a, b) != 3) return EXIT_FAILURE;

        if (sum(a, b) != 5) return EXIT_FAILURE;

        if (mul(a, b) != 6) return EXIT_FAILURE;

        if (div(a, b) != 0) return EXIT_FAILURE;

        return arr[len-1];
}
