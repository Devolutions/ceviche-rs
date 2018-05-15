#include <stdio.h>
#include <stdlib.h>
#include <tchar.h>
#include <Windows.h>

#include "NTService.h"

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
	NTService_OnServiceCreated OnServiceCreated;
	NTService_OnServiceDeleted OnServiceDeleted;
	NTService_OnServiceStart OnServiceStart;
	NTService_OnServiceStop OnServiceStop;
	NTService_OnServicePause OnServicePause;
	NTService_OnServiceContinue OnServiceContinue;
	NTService_OnServiceShutdown OnServiceShutdown;
	NTService_OnDeviceEvent OnDeviceEvent;
	NTService_OnHardwareProfileChange OnHardwareProfileChange;
	NTService_OnPowerEvent OnPowerEvent;
	NTService_OnSessionChange OnSessionChange;
	NTService_OnTimeChange OnTimeChange;
	NTService_OnTriggerEvent OnTriggerEvent;

	// Private members
	SERVICE_STATUS ssStatus;
	SERVICE_STATUS_HANDLE sshStatusHandle;
	DWORD dwControlsAccepted;
} NTServiceStruct;

#define ASSIGN_STRING(target, source) \
{ \
	if (target) free((void*) target); \
	target = (LPCTSTR)(source ? _tcsdup(source) : NULL); \
}

// Implementation is currently a singleton.
static NTServiceStruct* g_TheService = NULL;

static void NTService_UpdateControlsAccepted(NTService* service);
static void GetLastErrorText(LPTSTR lpBuffer, int nSize);

NTService* NTService_New(LPCTSTR serviceName, LPCTSTR displayName, LPCTSTR description)
{
	NTServiceStruct* self;

	// Implementation currently only supports 1 service.
	if (g_TheService)
		return NULL;

	self = (NTServiceStruct*) malloc(sizeof(NTServiceStruct));
	
	if (!self)
		return NULL;

	memset(self, 0, sizeof(NTServiceStruct));
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

	return (NTService*) self;
}

void NTService_Free(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;
	
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
LPCTSTR NTService_GetServiceName(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszServiceName;
}

void NTService_SetServiceName(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszServiceName, value);
}

LPCTSTR NTService_GetDisplayName(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszDisplayName;
}

void NTService_SetDisplayName(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszDisplayName, value);
}

LPCTSTR NTService_GetDescription(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszDescription;
}

void NTService_SetDescription(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszDescription, value);
}

DWORD NTService_GetDesiredAccess(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->dwDesiredAccess;
}

void NTService_SetDesiredAccess(NTService* service, DWORD value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->dwDesiredAccess = value;
}

DWORD NTService_GetServiceType(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->dwServiceType;
}

void NTService_SetServiceType(NTService* service, DWORD value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->dwServiceType = value;
}

DWORD NTService_GetStartType(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->dwStartType;
}

void NTService_SetStartType(NTService* service, DWORD value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->dwStartType = value;
}

DWORD NTService_GetErrorControl(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->dwErrorControl;
}

void NTService_SetErrorControl(NTService* service, DWORD value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->dwErrorControl = value;
}

LPCTSTR NTService_GetLoadOrderGroup(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszLoadOrderGroup;
}

void NTService_SetLoadOrderGroup(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszLoadOrderGroup, value);
}

LPCTSTR NTService_GetDependencies(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszDependencies;
}

void NTService_SetDependencies(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszDependencies, value);
}

LPCTSTR NTService_GetAccountName(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszAccountName;
}

void NTService_SetAccountName(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszAccountName, value);
}

LPCTSTR NTService_GetPassword(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->pszPassword;
}

void NTService_SetPassword(NTService* service, LPCTSTR value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	ASSIGN_STRING(self->pszPassword, value);
}

void* NTService_Getvalue(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	return self->lpvalue;
}

void NTService_Setvalue(NTService* service, void* value)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->lpvalue = value;
}

void NTService_SetOnServiceCreated(NTService* service, NTService_OnServiceCreated onServiceCreated)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceCreated = onServiceCreated;
}

void NTService_SetOnServiceDeleted(NTService* service, NTService_OnServiceDeleted onServiceDeleted)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceDeleted = onServiceDeleted;
}

void NTService_SetOnServiceStart(NTService* service, NTService_OnServiceStart onServiceStart)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceStart = onServiceStart;
}

void NTService_SetOnServiceStop(NTService* service, NTService_OnServiceStop onServiceStop)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceStop = onServiceStop;
}

void NTService_SetOnServicePause(NTService* service, NTService_OnServicePause onServicePause)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServicePause = onServicePause;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnServiceContinue(NTService* service, NTService_OnServiceContinue onServiceContinue)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceContinue = onServiceContinue;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnServiceShutdown(NTService* service, NTService_OnServiceShutdown onServiceShutdown)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnServiceShutdown = onServiceShutdown;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnDeviceEvent(NTService* service, NTService_OnDeviceEvent onDeviceEvent)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnDeviceEvent = onDeviceEvent;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnHardwareProfileChange(NTService* service, NTService_OnHardwareProfileChange onHardwareProfileChange)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnHardwareProfileChange = onHardwareProfileChange;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnPowerEvent(NTService* service, NTService_OnPowerEvent onPowerEvent)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnPowerEvent = onPowerEvent;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnSessionChange(NTService* service, NTService_OnSessionChange onSessionChange)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnSessionChange = onSessionChange;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnTimeChange(NTService* service, NTService_OnTimeChange onTimeChange)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnTimeChange = onTimeChange;

	NTService_UpdateControlsAccepted(service);
}

void NTService_SetOnTriggerEvent(NTService* service, NTService_OnTriggerEvent onTriggerEvent)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

	self->OnTriggerEvent = onTriggerEvent;

	NTService_UpdateControlsAccepted(service);
}


//
// Methods
//
bool NTService_CreateService(NTService* service)
{
	TCHAR szExeFileName[MAX_PATH];
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NTServiceStruct* self = (NTServiceStruct*) service;

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

bool NTService_DeleteService(NTService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NTServiceStruct* self = (NTServiceStruct*) service;

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

bool NTService_StartService(NTService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;
	NTServiceStruct* self = (NTServiceStruct*) service;

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

bool NTService_StopService(NTService* service)
{
	SC_HANDLE scManager;
	SC_HANDLE hService;
	bool success = false;

	NTServiceStruct* self = (NTServiceStruct*) service;

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

void NTService_ReportStatus(NTService* service, DWORD currentState, DWORD waitHint)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

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

static DWORD WINAPI NTService_ServiceCtrlHandlerEx(DWORD controlCode, DWORD eventType, void* eventData, void* value)
{
	NTService* service = (NTService*) g_TheService;
	DWORD retCode = ERROR_CALL_NOT_IMPLEMENTED;

	switch (controlCode)
	{
		case SERVICE_CONTROL_STOP:
			if (g_TheService->OnServiceStop)
			{
				NTService_ReportStatus(service, SERVICE_STOP_PENDING, 3000);
				g_TheService->OnServiceStop(service, value);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_PAUSE:
			if (g_TheService->OnServicePause)
			{
				NTService_ReportStatus(service, SERVICE_PAUSE_PENDING, 3000);
				g_TheService->OnServicePause(service, value);
				NTService_ReportStatus(service, SERVICE_PAUSED, 0);
				retCode = NO_ERROR;
			}
			break;

		case SERVICE_CONTROL_CONTINUE:
			if (g_TheService->OnServiceContinue)
			{
				NTService_ReportStatus(service, SERVICE_CONTINUE_PENDING, 3000);
				g_TheService->OnServiceContinue(service, value);
				NTService_ReportStatus(service, SERVICE_RUNNING, 0);
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
			NTService_ReportStatus(service, g_TheService->ssStatus.dwCurrentState, 0);
			retCode = NO_ERROR;
			break;

		default:
			break;
	}

	return retCode;
}

static void WINAPI NTService_ServiceMain(DWORD argc, LPTSTR* argv)
{
	g_TheService->sshStatusHandle = RegisterServiceCtrlHandlerEx(g_TheService->pszServiceName,
		                                                     NTService_ServiceCtrlHandlerEx,
		                                                     g_TheService->lpvalue);

	if (g_TheService->sshStatusHandle)
	{
		NTService_ReportStatus(g_TheService, SERVICE_START_PENDING, 3000);

		// Invoke the OnServiceStart callback.
		if (g_TheService->OnServiceStart)
		{
			g_TheService->OnServiceStart(g_TheService, g_TheService->lpvalue);
		}

		NTService_ReportStatus(g_TheService, SERVICE_STOPPED, 0);
	}
}

static bool NTService_StartDispatcher(NTService* service)
{
	SERVICE_TABLE_ENTRY dispatchTable[2];

	NTServiceStruct* self = (NTServiceStruct*) service;

	dispatchTable[0].lpServiceName = (LPTSTR) self->pszServiceName;
	dispatchTable[0].lpServiceProc = NTService_ServiceMain;

	dispatchTable[1].lpServiceName = NULL;
	dispatchTable[1].lpServiceProc = NULL;

	return StartServiceCtrlDispatcher(dispatchTable) ? true : false;
}

bool NTService_ProcessCommandLine(NTService* service, int argc, LPCTSTR* argv)
{
	bool (*pFunction)(NTService* s);
	LPCTSTR accountName = NULL;
	LPCTSTR password = NULL;
	bool success;

	NTServiceStruct* self = (NTServiceStruct*) service;

	// Default to the service dispatcher.
	pFunction = NTService_StartDispatcher;

	argc--;
	argv++;

	while (argc > 0)
	{
		if (argv[0][0] == _T('-'))
		{
			switch (argv[0][1])
			{
				case _T('i'):  // Install the service
					pFunction = NTService_CreateService;
					break;
				case _T('u'):  // Uninstall the service
					pFunction = NTService_DeleteService;
					break;
				case _T('s'):  // Start the service
					pFunction = NTService_StartService;
					break;
				case _T('k'):  // Kill the service
					pFunction = NTService_StopService;
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
		NTService_SetAccountName(service, accountName);
	}

	if (password)
	{
		NTService_SetPassword(service, password);
	}

	success = pFunction(service);

	if (success)
	{
		// Invoke the OnServiceCreated callback.
		if ((pFunction == NTService_CreateService) && (self->OnServiceCreated))
		{
			self->OnServiceCreated(service, self->lpvalue);
		}

		// Invoke the OnServiceDeleted callback.
		if ((pFunction == NTService_DeleteService) && (self->OnServiceDeleted))
		{
			self->OnServiceDeleted(service, self->lpvalue);
		}
	}

	return success;
}

//
// Local functions
//
static void NTService_UpdateControlsAccepted(NTService* service)
{
	NTServiceStruct* self = (NTServiceStruct*) service;

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
