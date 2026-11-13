#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

int32_t add(int32_t a, int32_t b) {
    return (a + b);
}

int main(void) {
    int32_t x = add(2, 3);
    printf("%d\n", x);
    int32_t y = 10;
    (y = (y + 1));
    printf("%d\n", y);
    if ((x > 4)) {
        printf("%s\n", "big");
    } else {
        printf("%s\n", "small");
    }
    int32_t i = 0;
    while ((i < 3)) {
        printf("%d\n", i);
        (i = (i + 1));
    }
    return 0;
}

