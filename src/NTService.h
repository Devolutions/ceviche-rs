#ifndef NTSERVICE_H
#define NTSERVICE_H

#include <stdbool.h>
#include <windows.h>

typedef void NTService;

// Callback definitions
typedef void (*NTService_OnServiceCreated)(NTService* service, void* context);
typedef void (*NTService_OnServiceDeleted)(NTService* service, void* context);
typedef void (*NTService_OnServiceStart)(NTService* service, void* context);
typedef void (*NTService_OnServiceStop)(NTService* service, void* context);
typedef void (*NTService_OnServicePause)(NTService* service, void* context);
typedef void (*NTService_OnServiceContinue)(NTService* service, void* context);
typedef void (*NTService_OnServiceShutdown)(NTService* service, void* context);
typedef void (*NTService_OnDeviceEvent)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnHardwareProfileChange)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnPowerEvent)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnSessionChange)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnTimeChange)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnTriggerEvent)(NTService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NTService_OnUserModeReboot)(NTService* service, DWORD eventType, void* eventData, void* context);

// Constructor/destructor
NTService* NTService_New(LPCTSTR serviceName, LPCTSTR displayName, LPCTSTR description);
void NTService_Free(NTService* service);

// Properties
LPCTSTR NTService_GetServiceName(NTService* service);
void NTService_SetServiceName(NTService* service, LPCTSTR value);

LPCTSTR NTService_GetDisplayName(NTService* service);
void NTService_SetDisplayName(NTService* service, LPCTSTR value);

LPCTSTR NTService_GetDescription(NTService* service);
void NTService_SetDescription(NTService* service, LPCTSTR value);

DWORD NTService_GetDesiredAccess(NTService* service);
void NTService_SetDesiredAccess(NTService* service, DWORD value);

DWORD NTService_GetServiceType(NTService* service);
void NTService_SetServiceType(NTService* service, DWORD value);

DWORD NTService_GetStartType(NTService* service);
void NTService_SetStartType(NTService* service, DWORD value);

DWORD NTService_GetErrorControl(NTService* service);
void NTService_SetErrorControl(NTService* service, DWORD value);

LPCTSTR NTService_GetLoadOrderGroup(NTService* service);
void NTService_SetLoadOrderGroup(NTService* service, LPCTSTR value);

LPCTSTR NTService_GetDependencies(NTService* service);
void NTService_SetDependencies(NTService* service, LPCTSTR value);

LPCTSTR NTService_GetAccountName(NTService* service);
void NTService_SetAccountName(NTService* service, LPCTSTR value);

LPCTSTR NTService_GetPassword(NTService* service);
void NTService_SetPassword(NTService* service, LPCTSTR value);

void* NTService_GetContext(NTService* service);
void NTService_SetContext(NTService* service, void* context);

//**********************************************************************
//
// Service Event Handlers
//
//   OnServiceCreated - Performs additional operations after a service
//      has been created (e.g., creating devices, initializing registry
//      values, etc.).
//
//   OnServiceDeleted - Performs additional operations after a service
//      has been deleted (e.g., deleting devices, cleaning up registry
//      values, etc.).
//
//   OnServiceStart - Called by ServiceMain as the main thread of
//      execution for the service.  This callback must not return
//      until the service is stopped.
//
//   OnServiceStop - Signals the service to stop execution.
//
//   OnServicePause - Signals the service to pause execution.
//
//   OnServiceContinue - Signals the service to continue execution.
//
//   OnServiceShutdown - Notifies the service of system shutdown.
//
//   OnDeviceEvent - Notifies the service of a device event.
//
//   OnHardwareProfileChange - Notifies the service of a hardware change.
//
//   OnPowerEvent - Notifies the service of a system power event.
//
//   OnSessionChange - Notifies the service of a session state change.
//
//   OnTimeChange - Notifies the service of a time change.
//
//   OnTriggerEvent - Notifies the service of a trigger event.
//
//**********************************************************************

void NTService_SetOnServiceCreated(NTService* service, NTService_OnServiceCreated onServiceCreated);
void NTService_SetOnServiceDeleted(NTService* service, NTService_OnServiceDeleted onServiceDeleted);
void NTService_SetOnServiceStart(NTService* service, NTService_OnServiceStart onServiceStart);
void NTService_SetOnServiceStop(NTService* service, NTService_OnServiceStop onServiceStop);
void NTService_SetOnServicePause(NTService* service, NTService_OnServicePause onServicePause);
void NTService_SetOnServiceContinue(NTService* service, NTService_OnServiceContinue onServiceContinue);
void NTService_SetOnServiceShutdown(NTService* service, NTService_OnServiceShutdown onServiceShutdown);
void NTService_SetOnDeviceEvent(NTService* service, NTService_OnDeviceEvent onDeviceEvent);
void NTService_SetOnHardwareProfileChange(NTService* service, NTService_OnHardwareProfileChange onHardwareProfileChange);
void NTService_SetOnPowerEvent(NTService* service, NTService_OnPowerEvent onPowerEvent);
void NTService_SetOnSessionChange(NTService* service, NTService_OnSessionChange onSessionChange);
void NTService_SetOnTimeChange(NTService* service, NTService_OnTimeChange onTimeChange);
void NTService_SetOnTriggerEvent(NTService* service, NTService_OnTriggerEvent onTriggerEvent);

bool NTService_ProcessCommandLine(NTService* service, int argc, LPCTSTR* argv);
bool NTService_CreateService(NTService* service);
bool NTService_DeleteService(NTService* service);
bool NTService_StartService(NTService* service);
bool NTService_StopService(NTService* service);
void NTService_ReportStatus(NTService* service, DWORD dwCurrentState, DWORD dwWaitHint);

#endif