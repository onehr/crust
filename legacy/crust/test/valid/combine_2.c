int fib(int a) {if (a == 0 || a == 1) {return a;} else {return fib(a - 1) + fib(a - 2);}}
int max(int a, int b) {return a > b ? a : b;}
int min(int a, int b) {return a < b ? a : b;}
int sum(int a, int b) {return a + b;}
int mul(int a, int b) {return a * b;}
int div(int a, int b) {return a / b;}

int EXIT_SUCCESS = 0;
int EXIT_FAILURE = 1;

int main() {
        int a = 2;
        int b = 3;
        int n = 10;

        if (fib(n) != 55) return EXIT_FAILURE;

        if (min(a, b) != 2) return EXIT_FAILURE;

        if (max(a, b) != 3) return EXIT_FAILURE;

        if (sum(a, b) != 5) return EXIT_FAILURE;

        if (mul(a, b) != 6) return EXIT_FAILURE;

        if (div(a, b) != 0) return EXIT_FAILURE;

        return EXIT_SUCCESS;
}
