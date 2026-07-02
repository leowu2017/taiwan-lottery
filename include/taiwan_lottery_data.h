#ifndef TAIWAN_LOTTERY_DATA_H
#define TAIWAN_LOTTERY_DATA_H

#ifdef __cplusplus
extern "C" {
#endif

/* Return codes for download_all */
#define TAIWAN_LOTTERY_OK 0
#define TAIWAN_LOTTERY_NULL_PATH 1
#define TAIWAN_LOTTERY_INVALID_PATH_UTF8 2
#define TAIWAN_LOTTERY_IO_ERROR 3
#define TAIWAN_LOTTERY_NETWORK_ERROR 4
#define TAIWAN_LOTTERY_PARSE_ERROR 5
#define TAIWAN_LOTTERY_NULL_DATASET_CODE 6
#define TAIWAN_LOTTERY_INVALID_DATASET_CODE_UTF8 7

int download_api_doc(const char* output_dir);
int download_dataset(const char* output_dir, const char* dataset_code);
int download_history_draw(const char* output_dir);
int download_all(const char* output_dir);

#ifdef __cplusplus
}
#endif

#endif
