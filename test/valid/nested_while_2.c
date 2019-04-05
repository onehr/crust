int main() {
        int a = 1;
        int b = 1;
        int ans = 0;
        while (a < 10) {
                while (b < 10) {
                        b = b + 1;
                        ans = ans + 1;
                }
                a = a + 1;
        }
        return ans;
}
