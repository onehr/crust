int main() {
    int sum = 0;
    for (int i = 0; i < 10;) {
        i = i + 1;
        if (i == 0 || i == 2 || i == 4 || i == 6 || i == 8)
            continue;
        sum = sum + i;
    }
    return sum;
}
