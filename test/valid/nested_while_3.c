int main() {
        int ans = 0;
        int a = 0;
        int b = 0;
        int c = 0;
        while (a < 4) {
                a = a + 1;
                while (b < 10) {
                        b = b + 1;
                        while (c < 100) {
                                ans = ans + 1;
                                c = c + 1;
                        }
                }
        }
        return ans;
}
