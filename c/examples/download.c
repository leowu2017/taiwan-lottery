#include <stdio.h>
#include <string.h>

#include <taiwan_lottery/download.h>

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s all [output_dir]\n", program);
    fprintf(stderr, "  %s api-doc [output_dir]\n", program);
    fprintf(stderr, "  %s history-draw [output_dir]\n", program);
    fprintf(stderr, "  %s history-draw-taiwan-lottery [output_dir]\n", program);
    fprintf(stderr, "  %s dataset <DATASET_CODE> [output_dir]\n", program);
}

static const char *status_to_message(int status) {
    switch (status) {
    case TAIWAN_LOTTERY_OK:
        return "success";
    case TAIWAN_LOTTERY_NULL_PATH:
        return "output_dir is null";
    case TAIWAN_LOTTERY_INVALID_PATH_UTF8:
        return "output_dir is not valid UTF-8";
    case TAIWAN_LOTTERY_IO_ERROR:
        return "I/O error";
    case TAIWAN_LOTTERY_NETWORK_ERROR:
        return "network error";
    case TAIWAN_LOTTERY_PARSE_ERROR:
        return "parse error";
    case TAIWAN_LOTTERY_NULL_DATASET_CODE:
        return "dataset_code is null";
    case TAIWAN_LOTTERY_INVALID_DATASET_CODE_UTF8:
        return "dataset_code is not valid UTF-8";
    default:
        return "unknown error";
    }
}

int main(int argc, char **argv) {
    const char *program = argc > 0 ? argv[0] : "download";
    const char *mode = argc > 1 ? argv[1] : "all";
    const char *default_output_dir = "data";
    const char *output_dir = default_output_dir;
    int status = TAIWAN_LOTTERY_OK;

    if (strcmp(mode, "all") == 0) {
        output_dir = argc > 2 ? argv[2] : default_output_dir;
        status = download_all(output_dir);
    } else if (strcmp(mode, "api-doc") == 0) {
        output_dir = argc > 2 ? argv[2] : default_output_dir;
        status = download_api_doc(output_dir);
    } else if (strcmp(mode, "history-draw") == 0) {
        output_dir = argc > 2 ? argv[2] : default_output_dir;
        status = download_history_draw(output_dir);
    } else if (strcmp(mode, "history-draw-taiwan-lottery") == 0) {
        output_dir = argc > 2 ? argv[2] : default_output_dir;
        status = download_history_draw_from_taiwan_lottery(output_dir);
    } else if (strcmp(mode, "dataset") == 0) {
        if (argc < 3) {
            print_usage(program);
            return 2;
        }
        output_dir = argc > 3 ? argv[3] : default_output_dir;
        status = download_dataset(output_dir, argv[2]);
    } else {
        print_usage(program);
        return 2;
    }

    if (status != TAIWAN_LOTTERY_OK) {
        fprintf(stderr, "Download failed (status=%d): %s\n", status, status_to_message(status));
        return 1;
    }

    printf("Download completed successfully.\n");
    return 0;
}
