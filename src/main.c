#include <stdio.h>

#include "NtService.h"
#include "NtService.c"

#include "NtService.h"

#define SERVICE_NAME "foobar"
#define DISPLAY_NAME "FooBar Service"
#define DESCRIPTION  "This is the FooBar service"

static HANDLE g_hEvent = 0;

void TestService_OnServiceCreated(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServiceCreated\n");
}

void TestService_OnServiceDeleted(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServiceDeleted\n");
}
	
void TestService_OnServiceStart(NtService* pNtService, LPVOID lpContext)
{
	int i;
	
	printf("OnServiceStart\n");

	g_hEvent = CreateEvent(NULL, TRUE, FALSE, NULL);

	for (i = 0; i < 5; i++)
	{
		NtService_ReportStatus(pNtService, SERVICE_START_PENDING, 1000);
		Sleep(1000);
	}

	NtService_ReportStatus(pNtService, SERVICE_RUNNING, 0);

	WaitForSingleObject(g_hEvent, INFINITE);

	NtService_ReportStatus(pNtService, SERVICE_STOP_PENDING, 3000);

	CloseHandle(g_hEvent);
}

void TestService_OnServiceStop(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServiceStop\n");

	SetEvent(g_hEvent);
}

void TestService_OnServicePause(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServicePause\n");
}

void TestService_OnServiceContinue(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServiceContinue\n");
}

void TestService_OnServiceShutdown(NtService* pNtService, LPVOID lpContext)
{
	printf("OnServiceShutdown\n");
}

void TestService_OnDeviceEvent(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnDeviceEvent\n");
}

void TestService_OnHardwareProfileChange(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnHardwareProfileChange\n");
}

void TestService_OnPowerEvent(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnPowerEvent\n");
}

void TestService_OnSessionChange(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnSessionChange\n");
}

void TestService_OnTimeChange(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnTimeChange\n");
}

void TestService_OnTriggerEvent(NtService* pNtService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnTriggerEvent\n");
}

int main_c(int argc, char** argv)
{
	NtService* pNtService;
	
	pNtService = NtService_New(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION);
	NtService_SetOnServiceCreated(pNtService, TestService_OnServiceCreated);
	NtService_SetOnServiceDeleted(pNtService, TestService_OnServiceDeleted);
	NtService_SetOnServiceStart(pNtService, TestService_OnServiceStart);
	NtService_SetOnServiceStop(pNtService, TestService_OnServiceStop);
	NtService_SetOnServicePause(pNtService, TestService_OnServicePause);
	NtService_SetOnServiceContinue(pNtService, TestService_OnServiceContinue);
	NtService_SetOnServiceShutdown(pNtService, TestService_OnServiceShutdown);
	NtService_SetOnDeviceEvent(pNtService, TestService_OnDeviceEvent);
	NtService_SetOnHardwareProfileChange(pNtService, TestService_OnHardwareProfileChange);
	NtService_SetOnPowerEvent(pNtService, TestService_OnPowerEvent);
	NtService_SetOnSessionChange(pNtService, TestService_OnSessionChange);
	NtService_SetOnTimeChange(pNtService, TestService_OnTimeChange);
	NtService_SetOnTriggerEvent(pNtService, TestService_OnTriggerEvent);
	NtService_ProcessCommandLine(pNtService, argc, argv);
	NtService_Free(pNtService);

    return 0;
}
