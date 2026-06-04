#include <stdint.h>
#include <windows.h>

extern double main(void);

int _start() {
    double result = main();
    int exit_code = (int)result;
    ExitProcess(exit_code);
    return exit_code;
}
