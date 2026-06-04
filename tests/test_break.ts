function main() {
    let sum = 0;
    for (let i = 0; i < 10; i = i + 1) {
        if (i == 5) {
            break;
        }
        sum = sum + i;
    }
    print(sum);
    return 0;
}
