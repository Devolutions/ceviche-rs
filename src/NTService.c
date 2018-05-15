#include <stdio.h>
#include <stdlib.h>
#include <tchar.h>
#include <Windows.h>

#include "NtService.h"

typedef struct
{
	// Public members
	// Various parameters to CreateService
	LPCTSTR pszServiceName;
	LPCTSTR pszDisplayName;
	LPCTSTR pszDescription;
	DWORD dwDesiredAccess;
	DWORD dwServiceType;
	DWORD dwStartType;
	DWORD dwErrorControl;
	LPCTSTR pszLoadOrderGroup;
	DWORD dwTagID;
	LPCTSTR pszDependencies;
	LPCTSTR pszAccountName;
	LPCTSTR pszPassword;
	LPVOID lpvalue;

	// Callbacks for various service control events
	NtService_OnServiceCreated OnServiceCreated;
	NtService_OnServiceDeleted OnServiceDeleted;
	NtService_OnServiceStart OnServiceStart;
	NtService_OnServiceStop OnServiceStop;
	NtService_OnServicePause OnServicePause;
	NtService_OnServiceContinue OnServiceContinue;
	NtService_OnServiceShutdown OnServiceShutdown;
	NtService_OnDeviceEvent OnDeviceEvent;
	NtService_OnHardwareProfileChange OnHardwareProfileChange;
	NtService_OnPowerEvent OnPowerEvent;
	NtService_OnSessionChange OnSessionChange;
	NtService_OnTimeChange OnTimeChange;
	NtService_OnTriggerEvent OnTriggerEvent;

	// Private members
	SERVICE_STATUS ssStatus;
	SERVICE_STATUS_HANDLE sshStatusHandle;
	DWORD dwControlsAccepted;
} NtServiceStruct;

#define ASSIGN_STRING(target, source) \
{ \
	if (target) free((void*) target); \
	target = (LPCTSTR)(source ? _tcsdup(source) : NULL); \
}

// Implementation is currently a singleton.
static NtServiceStruct* g_TheService = NULL;

static void NtService_UpdateControlsAccepted(NtService* service);
static void GetLastErrorText(LPTSTR lpBuffer, int nSize);

NtService* NtService_New(LPCTSTR serviceName, LPCTSTR displayName, LPCTSTR description)
{
	NtServiceStruct* self;

	printf("serviceName: %s displayName: %s description: %s\n",
		serviceName, displayName, description);

	// Implementation currently only supports 1 service.
	if (g_TheService)
		return NULL;

	self = (NtServiceStruct*) malloc(sizeof(NtServiceStruct));
	
	if (!self)
		return NULL;

	memset(self, 0, sizeof(NtServiceStruct));
	ASSIGN_STRING(self->pszServiceName, serviceName);
	ASSIGN_STRING(self->pszDisplayName, displayName);
	ASSIGN_STRING(self->pszDescription, description);
	self->dwDesiredAccess = SERVICE_ALL_ACCESS;
	self->dwServiceType = SERVICE_WIN32_OWN_PROCESS;
	self->dwStartType = SERVICE_AUTO_START;
	self->dwErrorControl = SERVICE_ERROR_NORMAL;

	self->ssStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
	self->ssStatus.dwCurrentState = SERVICE_STOPPED;
	
	self->dwControlsAccepted = SERVICE_ACCEPT_STOP;

	g_TheService = self;

	return (NtService*) self;
}

void NtService_Free(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;
	
	g_TheService = NULL;
	
	if (self->pszServiceName)
		free((void*) self->pszServiceName);

	if (self->pszDisplayName)
		free((void*) self->pszDisplayName);

	if (self->pszDescription)
		free((void*) self->pszDescription);

	if (self->pszLoadOrderGroup)
		free((void*) self->pszLoadOrderGroup);

	if (self->pszDependencies)
		free((void*) self->pszDependencies);

	if (self->pszAccountName)
		free((void*) self->pszAccountName);

	if (self->pszPassword);
		free((void*) self->pszPassword);

	free(self);
}

//
// Properties
//
LPCTSTR NtService_GetServiceName(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszServiceName;
}

void NtService_SetServiceName(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszServiceName, value);
}

LPCTSTR NtService_GetDisplayName(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszDisplayName;
}

void NtService_SetDisplayName(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszDisplayName, value);
}

LPCTSTR NtService_GetDescription(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszDescription;
}

void NtService_SetDescription(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszDescription, value);
}

DWORD NtService_GetDesiredAccess(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->dwDesiredAccess;
}

void NtService_SetDesiredAccess(NtService* service, DWORD value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->dwDesiredAccess = value;
}

DWORD NtService_GetServiceType(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->dwServiceType;
}

void NtService_SetServiceType(NtService* service, DWORD value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->dwServiceType = value;
}

DWORD NtService_GetStartType(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->dwStartType;
}

void NtService_SetStartType(NtService* service, DWORD value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->dwStartType = value;
}

DWORD NtService_GetErrorControl(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->dwErrorControl;
}

void NtService_SetErrorControl(NtService* service, DWORD value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->dwErrorControl = value;
}

LPCTSTR NtService_GetLoadOrderGroup(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszLoadOrderGroup;
}

void NtService_SetLoadOrderGroup(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszLoadOrderGroup, value);
}

LPCTSTR NtService_GetDependencies(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszDependencies;
}

void NtService_SetDependencies(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszDependencies, value);
}

LPCTSTR NtService_GetAccountName(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszAccountName;
}

void NtService_SetAccountName(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszAccountName, value);
}

LPCTSTR NtService_GetPassword(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->pszPassword;
}

void NtService_SetPassword(NtService* service, LPCTSTR value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	ASSIGN_STRING(self->pszPassword, value);
}

void* NtService_Getvalue(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	return self->lpvalue;
}

void NtService_Setvalue(NtService* service, void* value)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->lpvalue = value;
}

void NtService_SetOnServiceCreated(NtService* service, NtService_OnServiceCreated onServiceCreated)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceCreated = onServiceCreated;
}

void NtService_SetOnServiceDeleted(NtService* service, NtService_OnServiceDeleted onServiceDeleted)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceDeleted = onServiceDeleted;
}

void NtService_SetOnServiceStart(NtService* service, NtService_OnServiceStart onServiceStart)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceStart = onServiceStart;
}

void NtService_SetOnServiceStop(NtService* service, NtService_OnServiceStop onServiceStop)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceStop = onServiceStop;
}

void NtService_SetOnServicePause(NtService* service, NtService_OnServicePause onServicePause)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServicePause = onServicePause;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnServiceContinue(NtService* service, NtService_OnServiceContinue onServiceContinue)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceContinue = onServiceContinue;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnServiceShutdown(NtService* service, NtService_OnServiceShutdown onServiceShutdown)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnServiceShutdown = onServiceShutdown;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnDeviceEvent(NtService* service, NtService_OnDeviceEvent onDeviceEvent)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnDeviceEvent = onDeviceEvent;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnHardwareProfileChange(NtService* service, NtService_OnHardwareProfileChange onHardwareProfileChange)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnHardwareProfileChange = onHardwareProfileChange;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnPowerEvent(NtService* service, NtService_OnPowerEvent onPowerEvent)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnPowerEvent = onPowerEvent;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnSessionChange(NtService* service, NtService_OnSessionChange onSessionChange)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnSessionChange = onSessionChange;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnTimeChange(NtService* service, NtService_OnTimeChange onTimeChange)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnTimeChange = onTimeChange;

	NtService_UpdateControlsAccepted(service);
}

void NtService_SetOnTriggerEvent(NtService* service, NtService_OnTriggerEvent onTriggerEvent)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->OnTriggerEvent = onTriggerEvent;

	NtService_UpdateControlsAccepted(service);
}


//
// Methods
//
bool NtService_CreateService(NtService* service)
{
	TCHAR szExeFileName[MAX_PATH];
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NtServiceStruct* self = (NtServiceStruct*) service;

	if (GetModuleFileName(NULL, szExeFileName, MAX_PATH) == 0)
	{
		TCHAR szErrorText[512];
		GetLastErrorText(szErrorText, _countof(szErrorText));
		_tprintf(_T("Service '%s' creation failed - %s\n"), self->pszDisplayName, szErrorText);
		return false;
	}

	scManager = OpenSCManager(NULL, NULL, SC_MANAGER_ALL_ACCESS);
	
	if (scManager)
	{
		LPDWORD pdwTagID = NULL;

		// Attempt to create the service.
		if (((self->dwServiceType == SERVICE_KERNEL_DRIVER) || (self->dwServiceType == SERVICE_FILE_SYSTEM_DRIVER))
			&& ((self->dwStartType == SERVICE_BOOT_START) || (self->dwStartType == SERVICE_SYSTEM_START)))
		{
			pdwTagID = &self->dwTagID;
		}

		hService = CreateService(scManager, self->pszServiceName, self->pszDisplayName, self->dwDesiredAccess,
			                 self->dwServiceType, self->dwStartType, self->dwErrorControl, szExeFileName,
			                 self->pszLoadOrderGroup, pdwTagID, self->pszDependencies, self->pszAccountName,
			                 self->pszPassword);
		if (hService)
		{
			SERVICE_DESCRIPTION sd;

			// Set the description for the service.
			sd.lpDescription = (LPTSTR) self->pszDescription;
			ChangeServiceConfig2(hService, SERVICE_CONFIG_DESCRIPTION, &sd);

			_tprintf(_T("Service '%s' created.\n"), self->pszDisplayName);
			CloseServiceHandle(hService);
			success = true;
		}
		else
		{
			TCHAR szErrorText[512];
			GetLastErrorText(szErrorText, _countof(szErrorText));
			_tprintf(_T("CreateService failed - %s\n"), szErrorText);
		}

		CloseServiceHandle(scManager);
	}
	else
	{
		TCHAR szErrorText[512];
		GetLastErrorText(szErrorText, _countof(szErrorText));
		_tprintf(_T("OpenSCManager failed - %s\n"), szErrorText);
	}

	return success;
}

bool NtService_DeleteService(NtService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NtServiceStruct* self = (NtServiceStruct*) service;

	scManager = OpenSCManager(NULL, NULL, SC_MANAGER_ALL_ACCESS);
	
	if (!scManager)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenSCManager failed - %s\n"), errorText);
		return false;
	}

	hService = OpenService(scManager, self->pszServiceName, SERVICE_ALL_ACCESS);
	
	if (!hService)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenService failed - %s\n"), errorText);
		return false;
	}
	
	// Attempt to stop the service.
	if (ControlService(hService, SERVICE_CONTROL_STOP, &self->ssStatus))
	{
		_tprintf(_T("Stopping service '%s'."), self->pszDisplayName);
		Sleep(1000);
		
		while (QueryServiceStatus(hService, &self->ssStatus))
		{
			if (self->ssStatus.dwCurrentState == SERVICE_STOP_PENDING)
			{
				_tprintf(_T("."));
				Sleep(1000);
			}
			else
			{
				break;
			}
		}

		if (self->ssStatus.dwCurrentState == SERVICE_STOPPED)
		{
			_tprintf(_T("\nService '%s' stopped.\n"), self->pszDisplayName);
		}
		else
		{
			_tprintf(_T("\nService '%s' failed to stop.\n"), self->pszDisplayName);
		}
	}

	// Attempt to delete the service.
	if (DeleteService(hService))
	{
		success = true;
		_tprintf(_T("Service '%s' deleted.\n"), self->pszDisplayName);
	}
	else
	{
		_tprintf(_T("Service '%s' failed to delete.\n"), self->pszDisplayName);
	}

	CloseServiceHandle(hService);
	CloseServiceHandle(scManager);

	return success;
}

bool NtService_StartService(NtService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;
	NtServiceStruct* self = (NtServiceStruct*) service;

	scManager = OpenSCManager(NULL, NULL, SC_MANAGER_ALL_ACCESS);
	
	if (!scManager)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenSCManager failed - %s\n"), errorText);
		return false;
	}

	hService = OpenService(scManager, self->pszServiceName, SERVICE_ALL_ACCESS);

	if (!hService)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenService failed - %s\n"), errorText);
		return false;
	}

	// Attempt to start the service.
	if (StartService(hService, 0, NULL))
	{
		_tprintf(_T("Starting service '%s'."), self->pszDisplayName);
		Sleep(1000);

		while (QueryServiceStatus(hService, &self->ssStatus))
		{
			if (self->ssStatus.dwCurrentState == SERVICE_START_PENDING)
			{
				_tprintf(_T("."));
				Sleep(1000);
			}
			else
			{
				break;
			}
		}

		if (self->ssStatus.dwCurrentState == SERVICE_RUNNING)
		{
			success = true;
			_tprintf(_T("\nService '%s' started.\n"), self->pszDisplayName);
		}
		else
		{
			_tprintf(_T("\nService '%s' failed to start.\n"), self->pszDisplayName);
		}
	}

	CloseServiceHandle(hService);
	CloseServiceHandle(scManager);
	
	return success;
}

bool NtService_StopService(NtService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NtServiceStruct* self = (NtServiceStruct*) service;

	scManager = OpenSCManager(NULL, NULL, SC_MANAGER_ALL_ACCESS);
	
	if (!scManager)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenSCManager failed - %s\n"), errorText);
		return false;
	}

	hService = OpenService(scManager, self->pszServiceName, SERVICE_ALL_ACCESS);

	if (!hService)
	{
		TCHAR errorText[512];
		GetLastErrorText(errorText, _countof(errorText));
		_tprintf(_T("OpenService failed - %s\n"), errorText);
		return false;
	}

	// Attempt to stop the service.
	if (ControlService(hService, SERVICE_CONTROL_STOP, &self->ssStatus))
	{
		_tprintf(_T("Stopping service '%s'."), self->pszDisplayName);

		Sleep(1000);
		while (QueryServiceStatus(hService, &self->ssStatus))
		{
			if (self->ssStatus.dwCurrentState == SERVICE_STOP_PENDING)
			{
				_tprintf(_T("."));
				Sleep(1000);
			}
			else
			{
				break;
			}
		}

		if (self->ssStatus.dwCurrentState == SERVICE_STOPPED)
		{
			success = true;
			_tprintf(_T("\nService '%s' stopped.\n"), self->pszDisplayName);
		}
		else
		{
			_tprintf(_T("\nService '%s' failed to stop.\n"), self->pszDisplayName);
		}
	}

		CloseServiceHandle(hService);
		CloseServiceHandle(scManager);


	return success;
}

void NtService_ReportStatus(NtService* service, DWORD currentState, DWORD waitHint)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->ssStatus.dwControlsAccepted = (currentState == SERVICE_START_PENDING ? 0 : self->dwControlsAccepted);

	if (currentState != self->ssStatus.dwCurrentState)
	{
		self->ssStatus.dwCheckPoint = 0;
	}
	else
	{
		self->ssStatus.dwCheckPoint++;
	}

	self->ssStatus.dwCurrentState = currentState;
	self->ssStatus.dwWin32ExitCode = NO_ERROR;
	self->ssStatus.dwWaitHint = waitHint;

	SetServiceStatus(self->sshStatusHandle, &self->ssStatus);
}

static DWORD WINAPI NtService_ServiceCtrlHandlerEx(DWORD controlCode, DWORD eventType, void* eventData, void* value)
{
	NtService* service = (NtService*) g_TheService;
	DWORD retCode = ERROR_CALL_NOT_IMPLEMENTED;

	switch (controlCode)
	{
		case SERVICE_CONTROL_STOP:
			if (g_TheService->OnServiceStop)
			{
				NtService_ReportStatus(service, SERVICE_STOP_PENDING, 3000);
				g_TheService->OnServiceStop(service, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_PAUSE:
			if (g_TheService->OnServicePause)
			{
				NtService_ReportStatus(service, SERVICE_PAUSE_PENDING, 3000);
				g_TheService->OnServicePause(service, value);
				NtService_ReportStatus(service, SERVICE_PAUSED, 0);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_CONTINUE:
			if (g_TheService->OnServiceContinue)
			{
				NtService_ReportStatus(service, SERVICE_CONTINUE_PENDING, 3000);
				g_TheService->OnServiceContinue(service, value);
				NtService_ReportStatus(service, SERVICE_RUNNING, 0);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_SHUTDOWN:
			if (g_TheService->OnServiceShutdown)
			{
				g_TheService->OnServiceShutdown(service, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_DEVICEEVENT:
			if (g_TheService->OnDeviceEvent)
			{
				g_TheService->OnDeviceEvent(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_HARDWAREPROFILECHANGE:
			if (g_TheService->OnHardwareProfileChange)
			{
				g_TheService->OnHardwareProfileChange(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_POWEREVENT:
			if (g_TheService->OnPowerEvent)
			{
				g_TheService->OnPowerEvent(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_SESSIONCHANGE:
			if (g_TheService->OnSessionChange)
			{
				g_TheService->OnSessionChange(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_TIMECHANGE:
			if (g_TheService->OnTimeChange)
			{
				g_TheService->OnTimeChange(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_TRIGGEREVENT:
			if (g_TheService->OnTriggerEvent)
			{
				g_TheService->OnTriggerEvent(service, eventType, eventData, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_INTERROGATE:
			NtService_ReportStatus(service, g_TheService->ssStatus.dwCurrentState, 0);
			retCode = NO_ERROR;
			break;

		default:
			break;
	}

	return retCode;
}

static void WINAPI NtService_ServiceMain(DWORD argc, LPTSTR* argv)
{
	g_TheService->sshStatusHandle = RegisterServiceCtrlHandlerEx(g_TheService->pszServiceName,
		                                                     NtService_ServiceCtrlHandlerEx,
		                                                     g_TheService->lpvalue);

	if (g_TheService->sshStatusHandle)
	{
		NtService_ReportStatus(g_TheService, SERVICE_START_PENDING, 3000);

		// Invoke the OnServiceStart callback.
		if (g_TheService->OnServiceStart)
		{
			g_TheService->OnServiceStart(g_TheService, g_TheService->lpvalue);
		}

		NtService_ReportStatus(g_TheService, SERVICE_STOPPED, 0);
	}
}

static bool NtService_StartDispatcher(NtService* service)
{
	SERVICE_TABLE_ENTRY dispatchTable[2];

	NtServiceStruct* self = (NtServiceStruct*) service;

	dispatchTable[0].lpServiceName = (LPTSTR) self->pszServiceName;
	dispatchTable[0].lpServiceProc = NtService_ServiceMain;

	dispatchTable[1].lpServiceName = NULL;
	dispatchTable[1].lpServiceProc = NULL;

	return StartServiceCtrlDispatcher(dispatchTable) ? true : false;
}

bool NtService_ProcessCommandLine(NtService* service, int argc, LPCTSTR* argv)
{
	bool (*pFunction)(NtService* s);
	LPCTSTR accountName = NULL;
	LPCTSTR password = NULL;
	bool success;

	NtServiceStruct* self = (NtServiceStruct*) service;

	// Default to the service dispatcher.
	pFunction = NtService_StartDispatcher;

	argc--;
	argv++;

	while (argc > 0)
	{
		if (argv[0][0] == _T('-'))
		{
			switch (argv[0][1])
			{
				case _T('i'):  // Install the service
					pFunction = NtService_CreateService;
					break;
				case _T('u'):  // Uninstall the service
					pFunction = NtService_DeleteService;
					break;
				case _T('s'):  // Start the service
					pFunction = NtService_StartService;
					break;
				case _T('k'):  // Kill the service
					pFunction = NtService_StopService;
					break;
				case _T('a'):  // Account name
					accountName = argv[1];
					argc--;
					argv++;
					break;
				case _T('p'):  // Password
					password = argv[1];
					argc--;
					argv++;
					break;
				default:
					break;
			}
		}

		argc--;
		argv++;
	}

	if (accountName)
	{
		NtService_SetAccountName(service, accountName);
	}

	if (password)
	{
		NtService_SetPassword(service, password);
	}

	success = pFunction(service);

	if (success)
	{
		// Invoke the OnServiceCreated callback.
		if ((pFunction == NtService_CreateService) && (self->OnServiceCreated))
		{
			self->OnServiceCreated(service, self->lpvalue);
		}

		// Invoke the OnServiceDeleted callback.
		if ((pFunction == NtService_DeleteService) && (self->OnServiceDeleted))
		{
			self->OnServiceDeleted(service, self->lpvalue);
		}
	}

	return success;
}

//
// Local functions
//
static void NtService_UpdateControlsAccepted(NtService* service)
{
	NtServiceStruct* self = (NtServiceStruct*) service;

	self->dwControlsAccepted = SERVICE_ACCEPT_STOP;

	if (self->OnServicePause && self->OnServiceContinue)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_PAUSE_CONTINUE;
	}

	if (self->OnServiceShutdown)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_SHUTDOWN;
	}

	if (self->OnHardwareProfileChange)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_HARDWAREPROFILECHANGE;
	}

	if (self->OnPowerEvent)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_POWEREVENT;
	}

	if (self->OnSessionChange)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_SESSIONCHANGE;
	}

	if (self->OnTimeChange)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_TIMECHANGE;
	}

	if (self->OnTriggerEvent)
	{
		self->dwControlsAccepted |= SERVICE_ACCEPT_TRIGGEREVENT;
	}
}

static void GetLastErrorText(LPTSTR buffer, int size)
{
	ZeroMemory(buffer, size * sizeof(TCHAR));
	FormatMessage(FORMAT_MESSAGE_FROM_SYSTEM, NULL, GetLastError(), LANG_NEUTRAL, buffer, size, NULL);
}
