// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Tracing implementation internals
#include "verilated_vcd_c.h"
#include "Vtop__Syms.h"


void Vtop___024root__trace_chg_0_sub_0(Vtop___024root* vlSelf, VerilatedVcd::Buffer* bufp);

void Vtop___024root__trace_chg_0(void* voidSelf, VerilatedVcd::Buffer* bufp) {
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root__trace_chg_0\n"); );
    // Init
    Vtop___024root* const __restrict vlSelf VL_ATTR_UNUSED = static_cast<Vtop___024root*>(voidSelf);
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    if (VL_UNLIKELY(!vlSymsp->__Vm_activity)) return;
    // Body
    Vtop___024root__trace_chg_0_sub_0((&vlSymsp->TOP), bufp);
}

void Vtop___024root__trace_chg_0_sub_0(Vtop___024root* vlSelf, VerilatedVcd::Buffer* bufp) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root__trace_chg_0_sub_0\n"); );
    // Init
    uint32_t* const oldp VL_ATTR_UNUSED = bufp->oldp(vlSymsp->__Vm_baseCode + 1);
    // Body
    if (VL_UNLIKELY(vlSelf->__Vm_traceActivity[1U])) {
        bufp->chgBit(oldp+0,(vlSelf->top__DOT__u_tracker__DOT__ev_pending));
        bufp->chgSData(oldp+1,(vlSelf->top__DOT__u_tracker__DOT__ev_request_id),16);
        bufp->chgSData(oldp+2,(vlSelf->top__DOT__u_tracker__DOT__ev_page_id),16);
        bufp->chgSData(oldp+3,(vlSelf->top__DOT__u_tracker__DOT__ev_timestamp),16);
        bufp->chgCData(oldp+4,(vlSelf->top__DOT__u_tracker__DOT__ev_operation),3);
        bufp->chgBit(oldp+5,(vlSelf->top__DOT__u_tracker__DOT__page_valid));
        bufp->chgSData(oldp+6,(vlSelf->top__DOT__u_tracker__DOT__page_page_id),16);
        bufp->chgSData(oldp+7,(vlSelf->top__DOT__u_tracker__DOT__page_request_id),16);
        bufp->chgSData(oldp+8,(vlSelf->top__DOT__u_tracker__DOT__page_access_count),16);
        bufp->chgSData(oldp+9,(vlSelf->top__DOT__u_tracker__DOT__page_last_access),16);
        bufp->chgCData(oldp+10,(vlSelf->top__DOT__u_tracker__DOT__page_refcount),8);
        bufp->chgCData(oldp+11,(vlSelf->top__DOT__u_tracker__DOT__page_state),2);
        bufp->chgBit(oldp+12,(vlSelf->top__DOT__u_tracker__DOT__next_error));
        bufp->chgBit(oldp+13,(vlSelf->top__DOT__u_tracker__DOT__next_page_valid));
        bufp->chgSData(oldp+14,(vlSelf->top__DOT__u_tracker__DOT__next_page_page_id),16);
        bufp->chgSData(oldp+15,(vlSelf->top__DOT__u_tracker__DOT__next_page_request_id),16);
        bufp->chgSData(oldp+16,(vlSelf->top__DOT__u_tracker__DOT__next_page_access_count),16);
        bufp->chgSData(oldp+17,(vlSelf->top__DOT__u_tracker__DOT__next_page_last_access),16);
        bufp->chgCData(oldp+18,(vlSelf->top__DOT__u_tracker__DOT__next_page_refcount),8);
        bufp->chgCData(oldp+19,(vlSelf->top__DOT__u_tracker__DOT__next_page_state),2);
    }
    bufp->chgBit(oldp+20,(vlSelf->clk));
    bufp->chgBit(oldp+21,(vlSelf->rst));
    bufp->chgBit(oldp+22,(vlSelf->event_valid));
    bufp->chgSData(oldp+23,(vlSelf->request_id),16);
    bufp->chgSData(oldp+24,(vlSelf->page_id),16);
    bufp->chgSData(oldp+25,(vlSelf->event_timestamp),16);
    bufp->chgCData(oldp+26,(vlSelf->operation),3);
    bufp->chgBit(oldp+27,(vlSelf->resp_valid));
    bufp->chgBit(oldp+28,(vlSelf->resp_error));
    bufp->chgBit(oldp+29,(vlSelf->meta_valid));
    bufp->chgSData(oldp+30,(vlSelf->meta_page_id),16);
    bufp->chgSData(oldp+31,(vlSelf->meta_request_id),16);
    bufp->chgSData(oldp+32,(vlSelf->meta_access_count),16);
    bufp->chgSData(oldp+33,(vlSelf->meta_last_access),16);
    bufp->chgCData(oldp+34,(vlSelf->meta_refcount),8);
    bufp->chgCData(oldp+35,(vlSelf->meta_state),2);
    bufp->chgSData(oldp+36,(vlSelf->hw_timestamp),16);
}

void Vtop___024root__trace_cleanup(void* voidSelf, VerilatedVcd* /*unused*/) {
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root__trace_cleanup\n"); );
    // Init
    Vtop___024root* const __restrict vlSelf VL_ATTR_UNUSED = static_cast<Vtop___024root*>(voidSelf);
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    // Body
    vlSymsp->__Vm_activity = false;
    vlSymsp->TOP.__Vm_traceActivity[0U] = 0U;
    vlSymsp->TOP.__Vm_traceActivity[1U] = 0U;
}
