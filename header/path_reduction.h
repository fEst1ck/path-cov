#ifndef PATH_REDUCER_H
#define PATH_REDUCER_H

#include <stdint.h>

typedef struct PathReducer PathReducer;
typedef int32_t BlockID;
typedef int32_t FunID;

PathReducer* get_path_reducer(const void* top_level, int32_t k);
const char* reduce_path(const PathReducer* reducer, const BlockID* path, int32_t path_size, FunID entry_fun_id);
