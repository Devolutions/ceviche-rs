#ifndef NTSERVICE_H
#define NTSERVICE_H

#include <stdbool.h>
#include <windows.h>

typedef void NtService;

// Callback definitions
typedef void (*NtService_OnServiceCreated)(NtService* service, void* context);
typedef void (*NtService_OnServiceDeleted)(NtService* service, void* context);
typedef void (*NtService_OnServiceStart)(NtService* service, void* context);
typedef void (*NtService_OnServiceStop)(NtService* service, void* context);
typedef void (*NtService_OnServicePause)(NtService* service, void* context);
typedef void (*NtService_OnServiceContinue)(NtService* service, void* context);
typedef void (*NtService_OnServiceShutdown)(NtService* service, void* context);
typedef void (*NtService_OnDeviceEvent)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnHardwareProfileChange)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnPowerEvent)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnSessionChange)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnTimeChange)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnTriggerEvent)(NtService* service, DWORD eventType, void* eventData, void* context);
typedef void (*NtService_OnUserModeReboot)(NtService* service, DWORD eventType, void* eventData, void* context);

// Constructor/destructor
NtService* NtService_New(LPCTSTR serviceName, LPCTSTR displayName, LPCTSTR description);
void NtService_Free(NtService* service);

// Properties
LPCTSTR NtService_GetServiceName(NtService* service);
void NtService_SetServiceName(NtService* service, LPCTSTR value);

LPCTSTR NtService_GetDisplayName(NtService* service);
void NtService_SetDisplayName(NtService* service, LPCTSTR value);

LPCTSTR NtService_GetDescription(NtService* service);
void NtService_SetDescription(NtService* service, LPCTSTR value);

DWORD NtService_GetDesiredAccess(NtService* service);
void NtService_SetDesiredAccess(NtService* service, DWORD value);

DWORD NtService_GetServiceType(NtService* service);
void NtService_SetServiceType(NtService* service, DWORD value);

DWORD NtService_GetStartType(NtService* service);
void NtService_SetStartType(NtService* service, DWORD value);

DWORD NtService_GetErrorControl(NtService* service);
void NtService_SetErrorControl(NtService* service, DWORD value);

LPCTSTR NtService_GetLoadOrderGroup(NtService* service);
void NtService_SetLoadOrderGroup(NtService* service, LPCTSTR value);

LPCTSTR NtService_GetDependencies(NtService* service);
void NtService_SetDependencies(NtService* service, LPCTSTR value);

LPCTSTR NtService_GetAccountName(NtService* service);
void NtService_SetAccountName(NtService* service, LPCTSTR value);

LPCTSTR NtService_GetPassword(NtService* service);
void NtService_SetPassword(NtService* service, LPCTSTR value);

void* NtService_GetContext(NtService* service);
void NtService_SetContext(NtService* service, void* context);

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

void NtService_SetOnServiceCreated(NtService* service, NtService_OnServiceCreated onServiceCreated);
void NtService_SetOnServiceDeleted(NtService* service, NtService_OnServiceDeleted onServiceDeleted);
void NtService_SetOnServiceStart(NtService* service, NtService_OnServiceStart onServiceStart);
void NtService_SetOnServiceStop(NtService* service, NtService_OnServiceStop onServiceStop);
void NtService_SetOnServicePause(NtService* service, NtService_OnServicePause onServicePause);
void NtService_SetOnServiceContinue(NtService* service, NtService_OnServiceContinue onServiceContinue);
void NtService_SetOnServiceShutdown(NtService* service, NtService_OnServiceShutdown onServiceShutdown);
void NtService_SetOnDeviceEvent(NtService* service, NtService_OnDeviceEvent onDeviceEvent);
void NtService_SetOnHardwareProfileChange(NtService* service, NtService_OnHardwareProfileChange onHardwareProfileChange);
void NtService_SetOnPowerEvent(NtService* service, NtService_OnPowerEvent onPowerEvent);
void NtService_SetOnSessionChange(NtService* service, NtService_OnSessionChange onSessionChange);
void NtService_SetOnTimeChange(NtService* service, NtService_OnTimeChange onTimeChange);
void NtService_SetOnTriggerEvent(NtService* service, NtService_OnTriggerEvent onTriggerEvent);

bool NtService_ProcessCommandLine(NtService* service, int argc, LPCTSTR* argv);
bool NtService_CreateService(NtService* service);
bool NtService_DeleteService(NtService* service);
bool NtService_StartService(NtService* service);
bool NtService_StopService(NtService* service);
void NtService_ReportStatus(NtService* service, DWORD dwCurrentState, DWORD dwWaitHint);

#endif