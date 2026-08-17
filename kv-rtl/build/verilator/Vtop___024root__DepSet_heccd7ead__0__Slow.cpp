// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design implementation internals
// See Vtop.h for the primary calling header

#include "Vtop__pch.h"
#include "Vtop___024root.h"

VL_ATTR_COLD void Vtop___024root___eval_static(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_static\n"); );
}

VL_ATTR_COLD void Vtop___024root___eval_initial(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_initial\n"); );
    // Body
    vlSelf->__Vtrigprevexpr___TOP__clk__0 = vlSelf->clk;
}

VL_ATTR_COLD void Vtop___024root___eval_final(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_final\n"); );
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__stl(Vtop___024root* vlSelf);
#endif  // VL_DEBUG
VL_ATTR_COLD bool Vtop___024root___eval_phase__stl(Vtop___024root* vlSelf);

VL_ATTR_COLD void Vtop___024root___eval_settle(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_settle\n"); );
    // Init
    IData/*31:0*/ __VstlIterCount;
    CData/*0:0*/ __VstlContinue;
    // Body
    __VstlIterCount = 0U;
    vlSelf->__VstlFirstIteration = 1U;
    __VstlContinue = 1U;
    while (__VstlContinue) {
        if (VL_UNLIKELY((0x64U < __VstlIterCount))) {
#ifdef VL_DEBUG
            Vtop___024root___dump_triggers__stl(vlSelf);
#endif
            VL_FATAL_MT("/home/hk/Desktop/minor/metadata_rtl/kv-rtl/rtl/top.sv", 5, "", "Settle region did not converge.");
        }
        __VstlIterCount = ((IData)(1U) + __VstlIterCount);
        __VstlContinue = 0U;
        if (Vtop___024root___eval_phase__stl(vlSelf)) {
            __VstlContinue = 1U;
        }
        vlSelf->__VstlFirstIteration = 0U;
    }
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__stl(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___dump_triggers__stl\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VstlTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VstlTriggered.word(0U))) {
        VL_DBG_MSGF("         'stl' region trigger index 0 is active: Internal 'stl' trigger - first iteration\n");
    }
}
#endif  // VL_DEBUG

VL_ATTR_COLD void Vtop___024root___stl_sequent__TOP__0(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___stl_sequent__TOP__0\n"); );
    // Body
    vlSelf->meta_valid = vlSelf->top__DOT__u_tracker__DOT__page_valid;
    vlSelf->meta_page_id = vlSelf->top__DOT__u_tracker__DOT__page_page_id;
    vlSelf->meta_request_id = vlSelf->top__DOT__u_tracker__DOT__page_request_id;
    vlSelf->meta_access_count = vlSelf->top__DOT__u_tracker__DOT__page_access_count;
    vlSelf->meta_last_access = vlSelf->top__DOT__u_tracker__DOT__page_last_access;
    vlSelf->meta_refcount = vlSelf->top__DOT__u_tracker__DOT__page_refcount;
    vlSelf->meta_state = vlSelf->top__DOT__u_tracker__DOT__page_state;
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

VL_ATTR_COLD void Vtop___024root___eval_stl(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_stl\n"); );
    // Body
    if ((1ULL & vlSelf->__VstlTriggered.word(0U))) {
        Vtop___024root___stl_sequent__TOP__0(vlSelf);
        vlSelf->__Vm_traceActivity[1U] = 1U;
        vlSelf->__Vm_traceActivity[0U] = 1U;
    }
}

VL_ATTR_COLD void Vtop___024root___eval_triggers__stl(Vtop___024root* vlSelf);

VL_ATTR_COLD bool Vtop___024root___eval_phase__stl(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___eval_phase__stl\n"); );
    // Init
    CData/*0:0*/ __VstlExecute;
    // Body
    Vtop___024root___eval_triggers__stl(vlSelf);
    __VstlExecute = vlSelf->__VstlTriggered.any();
    if (__VstlExecute) {
        Vtop___024root___eval_stl(vlSelf);
    }
    return (__VstlExecute);
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__act(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___dump_triggers__act\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VactTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VactTriggered.word(0U))) {
        VL_DBG_MSGF("         'act' region trigger index 0 is active: @(posedge clk)\n");
    }
}
#endif  // VL_DEBUG

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtop___024root___dump_triggers__nba(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___dump_triggers__nba\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VnbaTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VnbaTriggered.word(0U))) {
        VL_DBG_MSGF("         'nba' region trigger index 0 is active: @(posedge clk)\n");
    }
}
#endif  // VL_DEBUG

VL_ATTR_COLD void Vtop___024root___ctor_var_reset(Vtop___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtop__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtop___024root___ctor_var_reset\n"); );
    // Body
    vlSelf->clk = VL_RAND_RESET_I(1);
    vlSelf->rst = VL_RAND_RESET_I(1);
    vlSelf->event_valid = VL_RAND_RESET_I(1);
    vlSelf->request_id = VL_RAND_RESET_I(16);
    vlSelf->page_id = VL_RAND_RESET_I(16);
    vlSelf->event_timestamp = VL_RAND_RESET_I(16);
    vlSelf->operation = VL_RAND_RESET_I(3);
    vlSelf->resp_valid = VL_RAND_RESET_I(1);
    vlSelf->resp_error = VL_RAND_RESET_I(1);
    vlSelf->meta_valid = VL_RAND_RESET_I(1);
    vlSelf->meta_page_id = VL_RAND_RESET_I(16);
    vlSelf->meta_request_id = VL_RAND_RESET_I(16);
    vlSelf->meta_access_count = VL_RAND_RESET_I(16);
    vlSelf->meta_last_access = VL_RAND_RESET_I(16);
    vlSelf->meta_refcount = VL_RAND_RESET_I(8);
    vlSelf->meta_state = VL_RAND_RESET_I(2);
    vlSelf->hw_timestamp = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__ev_pending = VL_RAND_RESET_I(1);
    vlSelf->top__DOT__u_tracker__DOT__ev_request_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__ev_page_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__ev_timestamp = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__ev_operation = VL_RAND_RESET_I(3);
    vlSelf->top__DOT__u_tracker__DOT__page_valid = VL_RAND_RESET_I(1);
    vlSelf->top__DOT__u_tracker__DOT__page_page_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__page_request_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__page_access_count = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__page_last_access = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__page_refcount = VL_RAND_RESET_I(8);
    vlSelf->top__DOT__u_tracker__DOT__page_state = VL_RAND_RESET_I(2);
    vlSelf->top__DOT__u_tracker__DOT__next_error = VL_RAND_RESET_I(1);
    vlSelf->top__DOT__u_tracker__DOT__next_page_valid = VL_RAND_RESET_I(1);
    vlSelf->top__DOT__u_tracker__DOT__next_page_page_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__next_page_request_id = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__next_page_access_count = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__next_page_last_access = VL_RAND_RESET_I(16);
    vlSelf->top__DOT__u_tracker__DOT__next_page_refcount = VL_RAND_RESET_I(8);
    vlSelf->top__DOT__u_tracker__DOT__next_page_state = VL_RAND_RESET_I(2);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__Vfuncout = VL_RAND_RESET_I(1);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__stored_id = VL_RAND_RESET_I(16);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__1__event_id = VL_RAND_RESET_I(16);
    vlSelf->__Vfunc_sat_dec_ref__2__Vfuncout = VL_RAND_RESET_I(8);
    vlSelf->__Vfunc_sat_dec_ref__2__value = VL_RAND_RESET_I(8);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__Vfuncout = VL_RAND_RESET_I(1);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__stored_id = VL_RAND_RESET_I(16);
    vlSelf->__Vfunc_top__DOT__u_tracker__DOT__page_id_match__3__event_id = VL_RAND_RESET_I(16);
    vlSelf->__Vfunc_sat_inc_u__4__Vfuncout = VL_RAND_RESET_I(16);
    vlSelf->__Vfunc_sat_inc_u__4__value = VL_RAND_RESET_I(16);
    vlSelf->__Vtrigprevexpr___TOP__clk__0 = VL_RAND_RESET_I(1);
    for (int __Vi0 = 0; __Vi0 < 2; ++__Vi0) {
        vlSelf->__Vm_traceActivity[__Vi0] = 0;
    }
}
