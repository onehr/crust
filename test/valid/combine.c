int main() {
        int ans = 0;
        for (int i = 0; i < 10; i = i + 1) {
                for (int j = 0; j < 10; j = j + 1) {
                        ans = ans + 1;
                }
        }
        while (1) {
                ans = ans + 1;
                if (ans > 120) break;
        }
        do {
                ans = ans + 2;
        } while (ans > 160);

        int b = ans + 1;
        int c = b + 1;
        int d = b + c - ans;
        int a = ans * 2;
        a = ans - 1;
        for (int i = 0; i < 10; i = i + 1)
                a = a + 1;
        a = -a;
        a = -a;
        a = a / 2;
        a = a * 3;

        if (a == 197) {
                return 10 + a;
        } else
                return 0 + a;
}
