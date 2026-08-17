// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design implementation internals
// See Vtop.h for the primary calling header

#include "Vtop__pch.h"
#include "Vtop___024root.h"

void Vtop___024root___eval_act(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_act\n"); );
}

VL_INLINE_OPT void Vtop___024root___nba_sequent__TOP__0(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___nba_sequent__TOP__0\n"); );
    // Init
    CData/*0:0*/ __Vdly__top__DOT__u_tracker__DOT__ev_pending;
    __Vdly__top__DOT__u_tracker__DOT__ev_pending = 0;
    // Body
    __Vdly__top__DOT__u_tracker__DOT__ev_pending = vlSelf->top__DOT__u_tracker__DOT__ev_pending;
    if (vlSelf->rst) {
        vlSelf->hw_timestamp = 0U;
        __Vdly__top__DOT__u_tracker__DOT__ev_pending = 0U;
        vlSelf->top__DOT__u_tracker__DOT__ev_request_id = 0U;
        vlSelf->top__DOT__u_tracker__DOT__ev_page_id = 0U;
        vlSelf->top__DOT__u_tracker__DOT__ev_timestamp = 0U;
        vlSelf->top__DOT__u_tracker__DOT__ev_operation = 0U;
        vlSelf->resp_valid = 0U;
        vlSelf->resp_error = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_valid = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_page_id = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_request_id = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_access_count = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_last_access = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_refcount = 0U;
        vlSelf->top__DOT__u_tracker__DOT__page_state = 0U;
    } else {
        vlSelf->hw_timestamp = (0xffffU & ((IData)(1U) 
                                           + (IData)(vlSelf->hw_timestamp)));
        vlSelf->resp_valid = 0U;
        vlSelf->resp_error = 0U;
        if (vlSelf->top__DOT__u_tracker__DOT__ev_pending) {
            vlSelf->resp_valid = 1U;
            vlSelf->resp_error = vlSelf->top__DOT__u_tracker__DOT__next_error;
            vlSelf->top__DOT__u_tracker__DOT__page_valid 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_valid;
            vlSelf->top__DOT__u_tracker__DOT__page_page_id 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_page_id;
            vlSelf->top__DOT__u_tracker__DOT__page_request_id 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_request_id;
            vlSelf->top__DOT__u_tracker__DOT__page_access_count 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_access_count;
            vlSelf->top__DOT__u_tracker__DOT__page_last_access 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_last_access;
            vlSelf->top__DOT__u_tracker__DOT__page_refcount 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_refcount;
            vlSelf->top__DOT__u_tracker__DOT__page_state 
                = vlSelf->top__DOT__u_tracker__DOT__next_page_state;
            __Vdly__top__DOT__u_tracker__DOT__ev_pending = 0U;
        } else if (vlSelf->event_valid) {
            vlSelf->top__DOT__u_tracker__DOT__ev_request_id 
                = vlSelf->request_id;
            vlSelf->top__DOT__u_tracker__DOT__ev_page_id 
                = vlSelf->page_id;
            vlSelf->top__DOT__u_tracker__DOT__ev_timestamp 
                = vlSelf->event_timestamp;
            vlSelf->top__DOT__u_tracker__DOT__ev_operation 
                = vlSelf->operation;
            __Vdly__top__DOT__u_tracker__DOT__ev_pending = 1U;
        }
    }
    vlSelf->top__DOT__u_tracker__DOT__ev_pending = __Vdly__top__DOT__u_tracker__DOT__ev_pending;
    vlSelf->meta_request_id = vlSelf->top__DOT__u_tracker__DOT__page_request_id;
    vlSelf->meta_access_count = vlSelf->top__DOT__u_tracker__DOT__page_access_count;
    vlSelf->meta_last_access = vlSelf->top__DOT__u_tracker__DOT__page_last_access;
    vlSelf->meta_refcount = vlSelf->top__DOT__u_tracker__DOT__page_refcount;
    vlSelf->meta_state = vlSelf->top__DOT__u_tracker__DOT__page_state;
    vlSelf->meta_valid = vlSelf->top__DOT__u_tracker__DOT__page_valid;
    vlSelf->meta_page_id = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
    if (vlSelf->top__DOT__u_tracker__DOT__ev_pending) {
        if ((1U & (~ ((IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation) 
                      >> 2U)))) {
            if ((1U & (~ ((IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation) 
                          >> 1U)))) {
                if ((1U & ((~ (IData)(vlSelf->top__DOT__u_tracker__DOT__page_valid)) 
                           | (~ ([&]() {
                                        vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id 
                                            = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                                        vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id 
                                            = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
                                        vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout 
                                            = ((IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id) 
                                               == (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id));
                                    }(), (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout)))))) {
                } else {
                }
            }
        }
    }
    if (vlSelf->top__DOT__u_tracker__DOT__ev_pending) {
        if ((1U & (~ ((IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation) 
                      >> 2U)))) {
            if ((2U & (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation))) {
                if ((1U & (~ (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation)))) {
                    if ((1U & ((~ (IData)(vlSelf->top__DOT__u_tracker__DOT__page_valid)) 
                               | (~ ([&]() {
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id 
                                                = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id 
                                                = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout 
                                                = ((IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id) 
                                                   == (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id));
                                        }(), (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout)))))) {
                    }
                }
            }
        }
    }
    if (vlSelf->top__DOT__u_tracker__DOT__ev_pending) {
        if ((1U & (~ ((IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation) 
                      >> 2U)))) {
            if ((2U & (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation))) {
                if ((1U & (~ (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation)))) {
                    if ((1U & ((~ (IData)(vlSelf->top__DOT__u_tracker__DOT__page_valid)) 
                               | (~ ([&]() {
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id 
                                                = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id 
                                                = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
                                            vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout 
                                                = ((IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id) 
                                                   == (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id));
                                        }(), (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout)))))) {
                    }
                }
            }
        }
    }
    vlSelf->top__DOT__u_tracker__DOT__next_error = 0U;
    vlSelf->top__DOT__u_tracker__DOT__next_page_valid 
        = vlSelf->top__DOT__u_tracker__DOT__page_valid;
    vlSelf->top__DOT__u_tracker__DOT__next_page_page_id 
        = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
    vlSelf->top__DOT__u_tracker__DOT__next_page_request_id 
        = vlSelf->top__DOT__u_tracker__DOT__page_request_id;
    vlSelf->top__DOT__u_tracker__DOT__next_page_access_count 
        = vlSelf->top__DOT__u_tracker__DOT__page_access_count;
    vlSelf->top__DOT__u_tracker__DOT__next_page_last_access 
        = vlSelf->top__DOT__u_tracker__DOT__page_last_access;
    vlSelf->top__DOT__u_tracker__DOT__next_page_refcount 
        = vlSelf->top__DOT__u_tracker__DOT__page_refcount;
    vlSelf->top__DOT__u_tracker__DOT__next_page_state 
        = vlSelf->top__DOT__u_tracker__DOT__page_state;
    if (vlSelf->top__DOT__u_tracker__DOT__ev_pending) {
        if ((4U & (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation))) {
            vlSelf->top__DOT__u_tracker__DOT__next_error = 1U;
        } else if ((2U & (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation))) {
            if ((1U & (IData)(vlSelf->top__DOT__u_tracker__DOT__ev_operation))) {
                if (vlSelf->top__DOT__u_tracker__DOT__page_valid) {
                    vlSelf->top__DOT__u_tracker__DOT__next_error = 1U;
                } else {
                    vlSelf->top__DOT__u_tracker__DOT__next_page_valid = 1U;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_page_id 
                        = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_request_id 
                        = vlSelf->top__DOT__u_tracker__DOT__ev_request_id;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_access_count = 0U;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_last_access = 0U;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_refcount = 1U;
                    vlSelf->top__DOT__u_tracker__DOT__next_page_state = 1U;
                }
            } else if ((1U & ((~ (IData)(vlSelf->top__DOT__u_tracker__DOT__page_valid)) 
                              | (~ ([&]() {
                                    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id 
                                        = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                                    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id 
                                        = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
                                    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout 
                                        = ((IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id) 
                                           == (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id));
                                }(), (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout)))))) {
                vlSelf->top__DOT__u_tracker__DOT__next_error = 1U;
            } else if ((0U == (IData)(vlSelf->top__DOT__u_tracker__DOT__page_refcount))) {
                vlSelf->top__DOT__u_tracker__DOT__next_error = 1U;
            } else {
                vlSelf->__Vfunc_sat_dec_ref__2__value 
                    = vlSelf->top__DOT__u_tracker__DOT__page_refcount;
                vlSelf->__Vfunc_sat_dec_ref__2__Vfuncout 
                    = ((0U == (IData)(vlSelf->__Vfunc_sat_dec_ref__2__value))
                        ? 0U : (0xffU & ((IData)(vlSelf->__Vfunc_sat_dec_ref__2__value) 
                                         - (IData)(1U))));
                vlSelf->top__DOT__u_tracker__DOT__next_page_refcount 
                    = vlSelf->__Vfunc_sat_dec_ref__2__Vfuncout;
            }
        } else if ((1U & ((~ (IData)(vlSelf->top__DOT__u_tracker__DOT__page_valid)) 
                          | (~ ([&]() {
                                vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id 
                                    = vlSelf->top__DOT__u_tracker__DOT__ev_page_id;
                                vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id 
                                    = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
                                vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout 
                                    = ((IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id) 
                                       == (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id));
                            }(), (IData)(vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout)))))) {
            vlSelf->top__DOT__u_tracker__DOT__next_error = 1U;
        } else {
            vlSelf->__Vfunc_sat_inc_u__4__value = vlSelf->top__DOT__u_tracker__DOT__page_access_count;
            vlSelf->__Vfunc_sat_inc_u__4__Vfuncout 
                = (0xffffU & ((0xffffU == (IData)(vlSelf->__Vfunc_sat_inc_u__4__value))
                               ? (IData)(vlSelf->__Vfunc_sat_inc_u__4__value)
                               : ((IData)(1U) + (IData)(vlSelf->__Vfunc_sat_inc_u__4__value))));
            vlSelf->top__DOT__u_tracker__DOT__next_page_last_access 
                = vlSelf->top__DOT__u_tracker__DOT__ev_timestamp;
            vlSelf->top__DOT__u_tracker__DOT__next_page_access_count 
                = vlSelf->__Vfunc_sat_inc_u__4__Vfuncout;
        }
    }
}

void Vtop___024root___eval_nba(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_nba\n"); );
    // Body
    if ((1ULL & vlSelf->__VnbaTriggered.word(0U))) {
        Vtop___024root___nba_sequent__TOP__0(vlSelf);
        vlSelf->__Vm_traceActivity[1U] = 1U;
    }
}

void Vtop___024root___eval_triggers__act(Vtop___024root* vlSelf);

bool Vtop___024root___eval_phase__act(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_phase__act\n"); );
    // Init
    VlTriggerVec<1> __VpreTriggered;
    CData/*0:0*/ __VactExecute;
    // Body
    Vtop___024root___eval_triggers__act(vlSelf);
    __VactExecute = vlSelf->__VactTriggered.any();
    if (__VactExecute) {
        __VpreTriggered.andNot(vlSelf->__VactTriggered, vlSelf->__VnbaTriggered);
        vlSelf->__VnbaTriggered.thisOr(vlSelf->__VactTriggered);
        Vtop___024root___eval_act(vlSelf);
    }
    return (__VactExecute);
}

bool Vtop___024root___eval_phase__nba(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_phase__nba\n"); );
    // Init
    CData/*0:0*/ __VnbaExecute;
    // Body
    __VnbaExecute = vlSelf->__VnbaTriggered.any();
    if (__VnbaExecute) {
        Vtop___024root___eval_nba(vlSelf);
        vlSelf->__VnbaTriggered.clear();
    }
    return (__VnbaExecute);
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__nba(Vtop___024root* vlSelf);
#endif  // VL_DEBUG
#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__act(Vtop___024root* vlSelf);
#endif  // VL_DEBUG

void Vtop___024root___eval(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval\n"); );
    // Init
    IData/*31:0*/ __VnbaIterCount;
    CData/*0:0*/ __VnbaContinue;
    // Body
    __VnbaIterCount = 0U;
    __VnbaContinue = 1U;
    while (__VnbaContinue) {
        if (VL_UNLIKELY((0x64U < __VnbaIterCount))) {
#ifdef VL_DEBUG
            Vtop___024root___dump_triggers__nba(vlSelf);
#endif
            VL_FATAL_MT("/home/hk/Desktop/minor/metadata_rtl/kv-rtl/rtl/top.sv", 5, "", "NBA region did not converge.");
        }
        __VnbaIterCount = ((IData)(1U) + __VnbaIterCount);
        __VnbaContinue = 0U;
        vlSelf->__VactIterCount = 0U;
        vlSelf->__VactContinue = 1U;
        while (vlSelf->__VactContinue) {
            if (VL_UNLIKELY((0x64U < vlSelf->__VactIterCount))) {
#ifdef VL_DEBUG
                Vtop___024root___dump_triggers__act(vlSelf);
#endif
                VL_FATAL_MT("/home/hk/Desktop/minor/metadata_rtl/kv-rtl/rtl/top.sv", 5, "", "Active region did not converge.");
            }
            vlSelf->__VactIterCount = ((IData)(1U) 
                                       + vlSelf->__VactIterCount);
            vlSelf->__VactContinue = 0U;
            if (Vtop___024root___eval_phase__act(vlSelf)) {
                vlSelf->__VactContinue = 1U;
            }
        }
        if (Vtop___024root___eval_phase__nba(vlSelf)) {
            __VnbaContinue = 1U;
        }
    }
}

#ifdef VL_DEBUG
void Vtop___024root___eval_debug_assertions(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_debug_assertions\n"); );
    // Body
    if (VL_UNLIKELY((vlSelf->clk & 0xfeU))) {
        Verilated::overWidthError("clk");}
    if (VL_UNLIKELY((vlSelf->rst & 0xfeU))) {
        Verilated::overWidthError("rst");}
    if (VL_UNLIKELY((vlSelf->event_valid & 0xfeU))) {
        Verilated::overWidthError("event_valid");}
    if (VL_UNLIKELY((vlSelf->operation & 0xf8U))) {
        Verilated::overWidthError("operation");}
}
#endif  // VL_DEBUG
