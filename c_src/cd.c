#include "cd.h"
#include <cdda_interface.h>
#include <cdio/cdio.h>
#include <cdio/cdtext.h>
#include <cdio/device.h>
#include <cdio/disc.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// It is not this functions job to free
char **get_devices() { return cdio_get_devices(cdio_os_driver); }

void free_devices(char **devices) {
  if (!devices)
    return;
  cdio_free_device_list(devices);
  return;
}

bool verify_audio(char *devicestr) {
  if (!devicestr || access(devicestr, F_OK) == -1)
    return false;
  cdrom_drive *drive = cdda_identify(devicestr, CDDA_MESSAGE_PRINTIT, NULL);
  if (!drive) {
    fprintf(stderr, "Failed to identify device %s", devicestr);
    return false;
  }
  cdda_verbose_set(drive, CDDA_MESSAGE_PRINTIT, CDDA_MESSAGE_PRINTIT);

  if (cdda_open(drive)) {
    fprintf(stderr, "Failed to open drive %s", devicestr);
    cdda_close(drive);
    return false;
  }
  cdda_close(drive);
  return true;
}

int track_num(char *devicestr) {
  if (!devicestr || access(devicestr, F_OK) == -1)
    return -1;
  cdrom_drive *drive = cdda_identify(devicestr, CDDA_MESSAGE_PRINTIT, NULL);
  if (!drive)
    return -1;
    
  if (cdda_open(drive)) {
    cdda_close(drive);
    return -1;
  }

  int num = cdda_tracks(drive);
  cdda_close(drive);
  return num;
}

TrackMeta get_track_metadata(char *devicestr, int track) {
    char *title = NULL;
    char *artist = NULL;
    char *genre = NULL;
    cdrom_drive *drive = NULL;

    if (track < 1 || !devicestr || *devicestr == '\0') {
        goto fallback;
    }

    drive = cdda_identify(devicestr, CDDA_MESSAGE_PRINTIT, NULL);
    if (!drive) goto fallback;
    
    if (cdda_open(drive)) {
        cdda_close(drive);
        goto fallback;
    }

    int num_tracks = cdda_tracks(drive);

    if (track > num_tracks || !cdda_track_audiop(drive, track)) {
        cdda_close(drive);
        goto fallback;
    }

    CdIo_t *device = cdio_open(devicestr, cdio_os_driver);
    if (device) {
        cdtext_t *cdtext = cdio_get_cdtext(device);
        if (cdtext) {
            if (cdtext_get_const(cdtext, CDTEXT_FIELD_TITLE, track)) {
                title = strdup(cdtext_get_const(cdtext, CDTEXT_FIELD_TITLE, track));
            }
            if (cdtext_get_const(cdtext, CDTEXT_FIELD_PERFORMER, track)) {
                artist = strdup(cdtext_get_const(cdtext, CDTEXT_FIELD_PERFORMER, track));
            } else if (cdtext_get_const(cdtext, CDTEXT_FIELD_COMPOSER, track)) {
                artist = strdup(cdtext_get_const(cdtext, CDTEXT_FIELD_COMPOSER, track));
            }
            if (cdtext_get_const(cdtext, CDTEXT_FIELD_GENRE, track)) {
                genre = strdup(cdtext_get_const(cdtext, CDTEXT_FIELD_GENRE, track));
            }
            cdtext_destroy(cdtext);
        }
        cdio_destroy(device);
    }
    
    cdda_close(drive);

fallback:
    // Safe fallbacks
    if (!title) title = strdup("Unknown title");
    if (!artist) artist = strdup("Unknown artist");
    if (!genre) genre = strdup("Unknown genre");

    return (TrackMeta){title, artist, genre};
}

void free_track_metadata(TrackMeta *meta) {
  free(meta->title);
  free(meta->artist);
  free(meta->genre);
  *meta = (TrackMeta){0};
}

int get_track_duration(char* devicestr, int track) {
  if (!devicestr || access(devicestr, F_OK) == -1) return -1;
  
  cdrom_drive* drive = cdda_identify(devicestr, CDDA_MESSAGE_PRINTIT, NULL);
  if (!drive) return -1;
  
  if (cdda_open(drive)) {
    cdda_close(drive);
    return -1;
  }
  
  long start = cdda_track_firstsector(drive, track);
  long end = cdda_track_lastsector(drive, track);
  int duration = (end - start + 1) / CDIO_CD_FRAMES_PER_SEC;
  
  cdda_close(drive);
  return duration;
}

// int main() {
//     char **devices = cdio_get_devices(cdio_os_driver);
//     if (devices == NULL || devices[0] == NULL) {
//         printf("No drives found\n");
//         return 1;
//     }
//
//     unsigned int device_count = 0;
//     while (devices[device_count] != NULL) {
//         printf("DEVICE %u: %s\n", device_count, devices[device_count]);
//         device_count++;
//     }
//
//     printf("Select a device index: ");
//     fflush(stdout);
//
//     char buf[128];
//     if (fgets(buf, sizeof(buf), stdin) == NULL) {
//         fprintf(stderr, "Error reading input\n");
//         cdio_free_device_list(devices);
//         return 1;
//     }
//
//     buf[strcspn(buf, "\n")] = '\0';
//
//     char *endptr;
//     errno = 0;
//     unsigned long idx = strtoul(buf, &endptr, 10);
//
//     bool invalid = 0;
//     if (endptr == buf) {
//         invalid = 1;
//     } else if (errno == ERANGE) {
//         invalid = 1;
//     } else {
//         char *p = endptr;
//         while (isspace((unsigned char)*p)) {
//             p++;
//         }
//         if (*p != '\0') {
//             invalid = 1;
//         }
//     }
//
//     if (invalid) {
//         fprintf(stderr, "Invalid number: %s\n", buf);
//         cdio_free_device_list(devices);
//         return 1;
//     }
//
//     if (idx >= device_count) {
//         fprintf(stderr, "Invalid index: %lu\n", idx);
//         cdio_free_device_list(devices);
//         return 1;
//     }
//
//     char *devicestr = devices[idx];
//     printf("Selected device: %s\n", devicestr);
//
//     cdrom_drive *drive = cdda_identify(devicestr, CDDA_MESSAGE_PRINTIT,
//     NULL); if (!drive) {
//       fprintf(stderr, "Failed to identify device %s", devicestr);
//       cdio_free_device_list(devices);
//       return 1;
//     }
//     cdda_verbose_set(drive, CDDA_MESSAGE_PRINTIT, CDDA_MESSAGE_PRINTIT);
//
//     if (cdda_open(drive)) {
//       fprintf(stderr, "Failed to open drive %s", devicestr);
//       cdda_close(drive);
//       cdio_free_device_list(devices);
//       return 1;
//     }
//
//     unsigned long tracks = cdda_tracks(drive);
//     printf("TOTAL TRACKS: %ld\n", tracks);
//
//     printf("Track  Start Sector  End Sector    Length (sectors)  Type\n");
//     printf("--------------------------------------------------------\n");
//
//     for (int track = 1; track <= tracks; track++) {
//         long start = cdda_track_firstsector(drive, track);
//         long end = cdda_track_lastsector(drive, track);
//         const char *type = cdda_track_audiop(drive, track) ? "AUDIO" :
//         "DATA";
//
//         printf("%-6d %-13ld %-13ld %-18ld %s\n",
//                track, start, end, (end - start + 1), type);
//     }
//
//     CdIo_t *p_cdio = cdio_open(devicestr, cdio_os_driver);
//
//     cdtext_t *cdtext = cdio_get_cdtext(p_cdio);
//     for (unsigned long track = 1; track <= tracks; track++) {
//       printf("Info for track %ld:\n\n", track);
//       printf("Title: %s\n", cdtext_get_const(cdtext, CDTEXT_FIELD_TITLE,
//       track)); printf("Genre: %s\n", cdtext_get_const(cdtext,
//       CDTEXT_FIELD_TITLE, track)); printf("Performer: %s\n",
//       cdtext_get_const(cdtext, CDTEXT_FIELD_PERFORMER, track));
//       printf("Songwriter: %s\n", cdtext_get_const(cdtext,
//       CDTEXT_FIELD_SONGWRITER, track)); printf("Composer: %s\n",
//       cdtext_get_const(cdtext, CDTEXT_FIELD_COMPOSER, track));
//       printf("Arranger: %s\n", cdtext_get_const(cdtext,
//       CDTEXT_FIELD_ARRANGER, track)); printf("Message: %s\n",
//       cdtext_get_const(cdtext, CDTEXT_FIELD_MESSAGE, track));
//     }
//     cdio_destroy(p_cdio);
//     cdda_close(drive);
//     cdio_free_device_list(devices);
//     return 0;
// }
