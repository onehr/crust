#define a 1
#define PI 3.1415926
#define EXIT_SUCCESS 1
#define AA TMP
#define TMP return EXIT_SUCCESS;

int main(void) {
	// AA should turn into return 1;
	AA
}

