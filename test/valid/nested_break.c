int main() {
    int ans = 0;
    for (int i = 0; i < 10; i = i + 1) {
            for (int j = 0; j < 10; j = j + 1) {
                    if (i == 0 || i == 2 || i == 4 || i == 6 || i == 8)
                            break;
                    else
                            ans = ans + i;
            }
    }
    return ans;
}
