#include <stdio.h>

#include "NTService.h"
#include "NTService.c"

#include "NTService.h"

#define SERVICE_NAME "foobar"
#define DISPLAY_NAME "FooBar Service"
#define DESCRIPTION  "This is the FooBar service"

static HANDLE g_hEvent = 0;

void TestService_OnServiceCreated(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServiceCreated\n");
}

void TestService_OnServiceDeleted(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServiceDeleted\n");
}
	
void TestService_OnServiceStart(NTService* pNTService, LPVOID lpContext)
{
	int i;
	
	printf("OnServiceStart\n");

	g_hEvent = CreateEvent(NULL, TRUE, FALSE, NULL);

	for (i = 0; i < 5; i++)
	{
		NTService_ReportStatus(pNTService, SERVICE_START_PENDING, 1000);
		Sleep(1000);
	}

	NTService_ReportStatus(pNTService, SERVICE_RUNNING, 0);

	WaitForSingleObject(g_hEvent, INFINITE);

	NTService_ReportStatus(pNTService, SERVICE_STOP_PENDING, 3000);

	CloseHandle(g_hEvent);
}

void TestService_OnServiceStop(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServiceStop\n");

	SetEvent(g_hEvent);
}

void TestService_OnServicePause(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServicePause\n");
}

void TestService_OnServiceContinue(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServiceContinue\n");
}

void TestService_OnServiceShutdown(NTService* pNTService, LPVOID lpContext)
{
	printf("OnServiceShutdown\n");
}

void TestService_OnDeviceEvent(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnDeviceEvent\n");
}

void TestService_OnHardwareProfileChange(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnHardwareProfileChange\n");
}

void TestService_OnPowerEvent(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnPowerEvent\n");
}

void TestService_OnSessionChange(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnSessionChange\n");
}

void TestService_OnTimeChange(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnTimeChange\n");
}

void TestService_OnTriggerEvent(NTService* pNTService, DWORD dwEventType, LPVOID lpEventData, LPVOID lpContext)
{
	printf("OnTriggerEvent\n");
}

int main_c(int argc, char** argv)
{
	NTService* pNTService;
	
	pNTService = NTService_New(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION);
	NTService_SetOnServiceCreated(pNTService, TestService_OnServiceCreated);
	NTService_SetOnServiceDeleted(pNTService, TestService_OnServiceDeleted);
	NTService_SetOnServiceStart(pNTService, TestService_OnServiceStart);
	NTService_SetOnServiceStop(pNTService, TestService_OnServiceStop);
	NTService_SetOnServicePause(pNTService, TestService_OnServicePause);
	NTService_SetOnServiceContinue(pNTService, TestService_OnServiceContinue);
	NTService_SetOnServiceShutdown(pNTService, TestService_OnServiceShutdown);
	NTService_SetOnDeviceEvent(pNTService, TestService_OnDeviceEvent);
	NTService_SetOnHardwareProfileChange(pNTService, TestService_OnHardwareProfileChange);
	NTService_SetOnPowerEvent(pNTService, TestService_OnPowerEvent);
	NTService_SetOnSessionChange(pNTService, TestService_OnSessionChange);
	NTService_SetOnTimeChange(pNTService, TestService_OnTimeChange);
	NTService_SetOnTriggerEvent(pNTService, TestService_OnTriggerEvent);
	NTService_ProcessCommandLine(pNTService, argc, argv);
	NTService_Free(pNTService);

    return 0;
}
