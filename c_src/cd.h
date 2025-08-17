#ifndef CD_H
#define CD_H

#include <stdbool.h>
#include <cdda_interface.h>

typedef struct TrackMeta {
  char *title;
  char *artist;
  char *genre;
}TrackMeta;

typedef struct CDStream {
    cdrom_drive *drive;
    int track;
    long first_sector;
    long current_sector;
    long last_sector;
} CDStream;

char **get_devices();

void free_devices(char **devices);

bool verify_audio(char *devicestr);

int track_num(char *devicestr);

TrackMeta get_track_metadata(char *devicestr, int track);

void free_track_metadata(TrackMeta *meta);

int get_track_duration(char* devicestr, int track);

CDStream *open_cd_stream(const char *devicestr, int track);

int read_cd_stream(CDStream *stream, void *buffer, int sectors);

bool seek_cd_stream(CDStream *stream, long sector);

#endif
