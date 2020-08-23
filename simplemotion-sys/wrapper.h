#include "SimpleMotionV2/user_options.h"
#include "SimpleMotionV2/simplemotion.h"

// NOTE: Redefined from simplemotion_defs.h as the `BV()` macro is not supported by bindgen
#define FLT_FOLLOWERROR 1<<1
#define FLT_OVERCURRENT 1<<2
#define FLT_COMMUNICATION 1<<3
#define FLT_ENCODER 1<<4
#define FLT_OVERTEMP 1<<5
#define FLT_UNDERVOLTAGE 1<<6
#define FLT_OVERVOLTAGE 1<<7
#define FLT_PROGRAM_OR_MEM 1<<8
#define FLT_HARDWARE 1<<9
#define FLT_OVERVELOCITY 1<<10
#define FLT_INIT 1<<11
#define FLT_MOTION 1<<12
#define FLT_RANGE 1<<13
#define FLT_PSTAGE_FORCED_OFF 1<<14
#define FLT_HOST_COMM_ERROR 1<<15
#define FLT_CONFIG 1<<16
//IO side macros
#define FLT_GC_COMM 1<<15
#define FLT_QUEUE_FULL FLT_PROGRAM_OR_MEM
#define FLT_SM485_ERROR FLT_COMMUNICATION
#define FLT_FIRMWARE FLT_PROGRAM_OR_MEM //non-recoverable program error
#define FLT_ALLOC FLT_PROGRAM_OR_MEM //memory etc allocation failed

#define STAT_TARGET_REACHED 1<<1
#define STAT_FERROR_RECOVERY 1<<2
#define STAT_RUN 1<<3
#define STAT_ENABLED 1<<4
#define STAT_FAULTSTOP 1<<5
#define STAT_FERROR_WARNING 1<<6
#define STAT_STO_ACTIVE 1<<7
#define STAT_SERVO_READY 1<<8
#define STAT_BRAKING 1<<10
#define STAT_HOMING 1<<11
#define STAT_INITIALIZED 1<<12
#define STAT_VOLTAGES_OK 1<<13
#define STAT_PERMANENT_STOP 1<<15
#define STAT_STANDING_STILL 1<<16
#define STAT_QUICK_STOP_ACTIVE 1<<17
#define STAT_SAFE_TORQUE_MODE_ACTIVE 1<<18
#define STAT_STANDBY 1<<19
