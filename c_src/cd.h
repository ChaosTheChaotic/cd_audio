#ifndef CD_H
#define CD_H

#include <stdbool.h>

typedef struct TrackMeta {
  char *title;
  char *artist;
  char *genre;
}TrackMeta;

char **get_devices();

void free_devices(char **devices);

bool verify_audio(char *devicestr);

int track_num(char *devicestr);

TrackMeta get_track_metadata(char *devicestr, int track);

void free_track_metadata(TrackMeta *meta);

int get_track_duration(char* devicestr, int track);

#endif
