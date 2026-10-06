#ifndef TAIWAN_LOTTERY_DOWNLOAD_H
#define TAIWAN_LOTTERY_DOWNLOAD_H

#include <taiwan_lottery/status.h>

#ifdef __cplusplus
extern "C" {
#endif

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
