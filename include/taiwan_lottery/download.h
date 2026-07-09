#ifndef TAIWAN_LOTTERY_DOWNLOAD_H
#define TAIWAN_LOTTERY_DOWNLOAD_H

#ifdef __cplusplus
extern "C" {
#endif

/* Return codes for all download_* C APIs */
#define TAIWAN_LOTTERY_OK 0
#define TAIWAN_LOTTERY_NULL_PATH 1
#define TAIWAN_LOTTERY_INVALID_PATH_UTF8 2
#define TAIWAN_LOTTERY_IO_ERROR 3
#define TAIWAN_LOTTERY_NETWORK_ERROR 4
#define TAIWAN_LOTTERY_PARSE_ERROR 5
#define TAIWAN_LOTTERY_NULL_DATASET_CODE 6
#define TAIWAN_LOTTERY_INVALID_DATASET_CODE_UTF8 7

/* Download the FinancialPlanning API docs JSON into output_dir. */
int download_api_doc(const char* output_dir);
/* Download one CSV dataset and any linked files referenced by that CSV. */
int download_dataset(const char* output_dir, const char* dataset_code);
/* Download history draw data from FinancialPlanning OpenData. */
int download_history_draw(const char* output_dir);
/* Download history draw data only from Taiwan Lottery yearly ZIP downloads. */
int download_history_draw_from_taiwan_lottery(const char* output_dir);
/* Download API docs and every dataset listed in those docs. */
int download_all(const char* output_dir);

#ifdef __cplusplus
}
#endif

#endif
