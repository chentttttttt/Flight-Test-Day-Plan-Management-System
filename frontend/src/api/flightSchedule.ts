import request from '@/utils/request'
import type { SubjectAttributeRecord } from '@/types/subjectAttribute'
export interface FlightSchedule extends SubjectAttributeRecord {
}

export function getFlights(): Promise<FlightSchedule[]> {
    return request({ url: '/flight/schedule', method: 'get' })
}

export function createFlight(data: any): Promise<FlightSchedule> {
    return request({ url: '/flight/schedule', method: 'post', data })
}

export function updateFlight(id: number, data: any): Promise<FlightSchedule> {
    return request({ url: `/flight/schedule/${id}`, method: 'put', data })
}

export function deleteFlight(id: number): Promise<void> {
    return request({ url: `/flight/schedule/${id}`, method: 'delete' })
}

export function autoScheduleFlights(params: any): Promise<{ flights: FlightSchedule[] }> {
    return request({ url: '/flight/schedule/auto', method: 'post', data: params })
}