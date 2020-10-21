#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv)
{
	// If we didn't get an argument, return err
	if (argc != 2)
	{
		fprintf(stderr, "Usage:\n\t./extract_data <file-name>");
		return EXIT_FAILURE;
	}

	// Open input and output files
	FILE *in = fopen(argv[1], "rb");
	FILE *out = fopen("output.bin", "wb");

	// The buffer for reading
	char buffer[2048];

	// Bytes read
	size_t bytes_read = 0;

	// Input size
	fseek(in, 0, SEEK_END);
	size_t input_size = ftell(in);
	fseek(in, 0, SEEK_SET);

	// Output size
	size_t output_size = 2048 * (input_size / 2352);

	// Read until we reach EOF
	while (bytes_read < output_size)
	{
		// Ignore header
		fseek(in, 24, SEEK_CUR);

		// Read data
		fread(buffer, sizeof(char), 2048, in);
		fwrite(buffer, sizeof(char), 2048, out);
		bytes_read += 2048;

		// Ignore footer
		fseek(in, 280, SEEK_CUR);
	}

	// Return success
	return EXIT_SUCCESS;
}
