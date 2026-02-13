// Backfill definitions for PipeWire 1.0.8+ symbols.
// Those are forward-compatible enum values and structs that aren't present in
// system pipewire headers for v0.3.7, but are required features for Discord.

#ifndef DISCORD_BACKFILLS_H
#define DISCORD_BACKFILLS_H

#include <pipewire/version.h>
#include <stdint.h>
#include <threads.h>

// PropertyFlags: macros, so #ifndef works
#ifndef SPA_POD_PROP_FLAG_HINT_DICT
#define SPA_POD_PROP_FLAG_HINT_DICT (1u << 2)
#endif

#ifndef SPA_POD_PROP_FLAG_MANDATORY
#define SPA_POD_PROP_FLAG_MANDATORY (1u << 3)
#endif

#ifndef SPA_POD_PROP_FLAG_DONT_FIXATE
#define SPA_POD_PROP_FLAG_DONT_FIXATE (1u << 4)
#endif

// Enum values and structs require version check
#if !PW_CHECK_VERSION(1, 0, 8)

#define SPA_META_SyncTimeline 9

struct spa_meta_sync_timeline {
  uint32_t flags;
  uint32_t padding;
  uint64_t acquire_point;
  uint64_t release_point;
};

#define SPA_DATA_SyncObj 5
#define SPA_PARAM_BUFFERS_metaType 7

#endif // !PW_CHECK_VERSION(1,0,8)

#endif // DISCORD_BACKFILLS_H
